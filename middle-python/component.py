import wit_world
import componentize_py_async_support

from componentize_py_async_support.streams import StreamReader, StreamWriter
from wit_world import exports
from wit_world.imports import transformer


class Transformer(exports.Transformer):
    async def transform(self, stream: StreamReader[str]) -> StreamReader[str]:
        return map_stream(await transformer.transform(map_stream(stream, lambda v: f"🐍{v}")), lambda v: f"{v}🐍")


def map_stream(stream: StreamReader[str], fun: Callable[[str], str]) -> StreamReader[str]:
    tx, rx = wit_world.string_stream()
    componentize_py_async_support.spawn(pipe_stream(stream, tx, fun))
    return rx


async def pipe_stream(rx: StreamReader[str], tx: StreamWriter[str], fun: Callable[[str], str]) -> None:
    with rx, tx:
        while not rx.writer_dropped:
            values = await rx.read(8)
            await tx.write_all(list(map(fun, values)))
