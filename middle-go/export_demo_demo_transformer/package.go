package export_demo_demo_transformer

import (
	"fmt"
	. "go.bytecodealliance.org/pkg/wit/types"
	transformer "wit_component/demo_demo_transformer"
)

func Transform(stream *StreamReader[string]) *StreamReader[string] {
	return mapStream(
		transformer.Transform(
			mapStream(stream, func(v string) string { return fmt.Sprintf("ʕ◔ϖ◔ʔ%v", v) }),
		),
		func(v string) string { return fmt.Sprintf("%vʕ◔ϖ◔ʔ", v) },
	)
}

func mapStream(stream *StreamReader[string], fun func(string) string) *StreamReader[string] {
	tx, rx := transformer.MakeStreamString()

	go func() {
		defer stream.Drop()
		defer tx.Drop()
		buffer := make([]string, 8)
		for !stream.WriterDropped() {
			readCount := stream.Read(buffer)
			for _, value := range buffer[:readCount] {
				tx.WriteAll([]string{fun(value)})
			}
		}
	}()

	return rx
}
