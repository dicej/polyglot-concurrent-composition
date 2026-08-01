package export_demo_demo_line_count

import (
	"fmt"
	. "go.bytecodealliance.org/pkg/wit/types"
	"net/url"
	"strings"
	lineCount "wit_component/demo_demo_line_count"
	client "wit_component/wasi_http_client"
	. "wit_component/wasi_http_types"
)

func CountLines(urls []string) (*StreamReader[lineCount.LineCount], *FutureReader[Result[Unit, ErrorCode]]) {
	channel := make(chan Result[lineCount.LineCount, ErrorCode])
	deferred := make([]string, 0)
	for _, url := range urls {
		if strings.Contains(url, "://go.dev") {
			go func() {
				channel <- retrieve(url)
			}()
		} else {
			deferred = append(deferred, url)
		}
	}

	go func() {
		stream, future := lineCount.CountLines(deferred)
		defer stream.Drop()
		defer future.Drop()

		buffer := make([]lineCount.LineCount, 1)
		for !stream.WriterDropped() {
			count := stream.Read(buffer)
			for _, value := range buffer[:count] {
				value.Deferrers = append(value.Deferrers, "go")
				channel <- Ok[lineCount.LineCount, ErrorCode](value)
			}
		}

		result := future.Read()
		if result.IsErr() {
			channel <- Err[lineCount.LineCount, ErrorCode](result.Err())
		}
	}()

	streamTx, streamRx := lineCount.MakeStreamLineCount()
	futureTx, futureRx := MakeFutureResultUnitErrorCode()

	go func() {
		for i := 0; i < len(urls); i++ {
			result := (<-channel)
			if result.IsOk() {
				streamTx.WriteAll([]lineCount.LineCount{result.Ok()})
			} else {
				futureTx.Write(Err[Unit, ErrorCode](result.Err()))
				break
			}
		}
	}()

	return streamRx, futureRx
}

func retrieve(urlString string) Result[lineCount.LineCount, ErrorCode] {
	parsed, err := url.Parse(urlString)
	if err != nil {
		return Err[lineCount.LineCount, ErrorCode](MakeErrorCodeInternalError(Some(err.Error())))
	}

	var scheme Scheme
	switch parsed.Scheme {
	case "http":
		scheme = MakeSchemeHttp()
	case "https":
		scheme = MakeSchemeHttps()
	default:
		scheme = MakeSchemeOther(parsed.Scheme)
	}

	request, send := RequestNew(
		MakeFields(),
		None[*StreamReader[uint8]](),
		trailersFuture(),
		None[*RequestOptions](),
	)
	send.Drop()
	request.SetScheme(Some(scheme)).Ok()
	request.SetAuthority(Some(parsed.Host)).Ok()
	request.SetPathWithQuery(Some(parsed.Path)).Ok()

	result := client.Send(request)
	if result.IsOk() {
		response := result.Ok()
		status := response.GetStatusCode()
		if status < 200 || status > 299 {
			return Err[lineCount.LineCount, ErrorCode](
				MakeErrorCodeInternalError(Some(fmt.Sprintf("unexpected status: %v", status))),
			)
		}

		rx, trailers := ResponseConsumeBody(response, unitFuture())
		trailers.Drop()
		defer rx.Drop()

		buffer := make([]uint8, 16*1024)
		count := uint64(0)
		for !rx.WriterDropped() {
			readCount := rx.Read(buffer)
			for _, value := range buffer[:readCount] {
				if value == 10 /*ascii newline*/ {
					count += 1
				}
			}
		}
		return Ok[lineCount.LineCount, ErrorCode](lineCount.LineCount{
			Url:       urlString,
			Count:     count,
			Retriever: "go",
			Deferrers: []string{},
		})
	} else {
		return Err[lineCount.LineCount, ErrorCode](result.Err())
	}
}

func trailersFuture() *FutureReader[Result[Option[*Fields], ErrorCode]] {
	tx, rx := MakeFutureResultOptionFieldsErrorCode()
	go tx.Write(Ok[Option[*Fields], ErrorCode](None[*Fields]()))
	return rx
}

func unitFuture() *FutureReader[Result[Unit, ErrorCode]] {
	tx, rx := MakeFutureResultUnitErrorCode()
	go tx.Write(Ok[Unit, ErrorCode](Unit{}))
	return rx
}
