import { transform } from "demo:demo/transformer"
import * as witWorld from "wit-world"

export const demoDemoTransformer = {
    transform: async function(stream) {
        return map(await transform(map(stream, (v) => `🐒${v}`)), (v) => `${v}🐒`)
    }
}

function map(stream, fun) {
    const [tx, rx] = witWorld.stringStream()
    pipe(stream, tx, fun)
    return rx
}

async function pipe(rx, tx, fun) {
    using _rx = rx, _tx = tx
    while (!rx.writerDropped) {
        const values = await rx.read(8)
        await tx.writeAll(values.map(fun))
    }
}
