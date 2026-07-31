import asyncio
import wit_world
import componentize_py_async_support

from typing import Any, Optional
from componentize_py_types import Ok, Err, Result
from componentize_py_async_support.streams import StreamReader, StreamWriter
from componentize_py_async_support.futures import FutureReader, FutureWriter
from wit_world import exports
from wit_world.imports import client
from wit_world.imports.line_count import LineCount as LineCountRecord, count_lines
from wit_world.imports.wasi_http_types import (
    Scheme,
    Scheme_Http,
    Scheme_Https,
    Scheme_Other,
    Request,
    Response,
    Fields,
    ErrorCode,
    ErrorCode_InternalError,
)
from urllib import parse


class LineCount(exports.LineCount):
    async def count_lines(self, urls: list[str]) -> tuple[StreamReader[LineCountRecord], FutureReader[Result[None, ErrorCode]]]:
        tasks = []
        deferred_futures = {}
        for url in urls:
            if "://www.python.org" in url:
                tasks.append(asyncio.ensure_future(retrieve(url)))
            else:
                future = asyncio.get_event_loop().create_future()
                deferred_futures[url] = future
                tasks.append(asyncio.ensure_future(future))

        componentize_py_async_support.spawn(defer(deferred_futures))

        stream_tx, stream_rx = wit_world.line_count_line_count_stream()
        future_tx, future_rx = wit_world.result_unit_wasi_http_types_error_code_future(lambda: Ok(None))

        componentize_py_async_support.spawn(feed(stream_tx, future_tx, tasks))

        return stream_rx, future_rx


async def feed(stream_tx: StreamWriter[LineCountRecord], future_tx: FutureWriter[Result[None, ErrorCode]], tasks: list[Any]) -> None:
    with stream_tx, future_tx:
        for future in asyncio.as_completed(tasks):
            print("completed")
            try:
                result = await future
                print(f"completed `{result.url}`")
                await stream_tx.write_all([result])
            except Err as e:
                await future_tx.write(e)
                break

    
async def retrieve(url: str) -> LineCountRecord:
    print(f"retrieving `{url}`")
    url_parsed = parse.urlparse(url)

    match url_parsed.scheme:
        case "http":
            scheme: Scheme = Scheme_Http()
        case "https":
            scheme = Scheme_Https()
        case _:
            scheme = Scheme_Other(url_parsed.scheme)

    request = Request.new(Fields(), None, trailers_future(), None)[0]
    request.set_scheme(scheme)
    request.set_authority(url_parsed.netloc)
    request.set_path_with_query(url_parsed.path)

    print(f"sending request to `{url}`")
    response = await client.send(request)
    status = response.get_status_code()
    if status < 200 or status > 299:
        raise Err(ErrorCode_InternalError(f"unexpected status for URL {url}: {status}"))

    print(f"consuming body for `{url}`")
    rx = Response.consume_body(response, unit_future())[0]

    count = 0
    with rx:
        while not rx.writer_dropped:
            chunk = await rx.read(16 * 1024)
            count += chunk.count(b'\n')

    print(f"retrieved `{url}`")
    return LineCountRecord(url, count, "python", [])


async def defer(futures: dict[str, Any]) -> None:
    print(f"deferring `{list(futures.keys())}`")
    stream, future = await count_lines(list(futures.keys()))

    with stream, future:
        while not stream.writer_dropped:
            values = await stream.read(1)
            for value in values:
                print(f"deferred `{value.url}`")
                value.deferrers.append("python")
                futures.pop(value.url).set_result(value)

        result = await future.read()
        if isinstance(result, Err):
            for promise in futures.values():
                promise.set_exception(result)

    print("exit defer")


def trailers_future() -> FutureReader[Result[Optional[Fields], ErrorCode]]:
    return wit_world.result_option_wasi_http_types_fields_wasi_http_types_error_code_future(lambda: Ok(None))[1]
                

def unit_future() -> FutureReader[Result[None, ErrorCode]]:
    return wit_world.result_unit_wasi_http_types_error_code_future(lambda: Ok(None))[1]
