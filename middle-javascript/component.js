import { countLines } from "demo:demo/line-count"
import { Request, Response, Fields } from "wasi:http/types@0.3.0"
import * as client from "wasi:http/client@0.3.0"
import * as witWorld from "wit-world"

const encoder = new TextEncoder()

export const demoDemoLineCount = {
    countLines: async function(urls) {
        const tasks = []
        const promises = {}
        for (const url of urls) {
            if (url.includes("://tc39.es")) {
                tasks.push([url, retrieve(url)])
            } else {
                tasks.push([url, new Promise((resolve, reject) => {
                    promises[url] = { resolve, reject }
                })])
            }
        }

        defer(promises) 

        const [streamTx, streamRx] = witWorld.demoDemoLineCountLineCountStream()
        const [futureTx, futureRx] = witWorld.resultUnitWasiHttpTypes030ErrorCodeFuture(() => { return { tag: "ok" } })

        feed(streamTx, futureTx, tasks)

        return [streamRx, futureRx]
    }
}

async function feed(streamTx, futureTx, tasks) {
    using _streamTx = streamTx, _futureTx = futureTx
    try {
        while (tasks.length > 0) {
            const result = await Promise.race(tasks.map(([_, v]) => v))
            tasks = tasks.filter(([k, _]) => k !== result.url)
            await streamTx.writeAll([result])
        }
    } catch (error) {
        await futureTx.write(error)
    }
}

async function retrieve(url) {
    // TODO: use a proper URL parser
    const schemeDelimiter = url.indexOf("://")
    if (schemeDelimiter === -1) {
        throw { tag: "err", val: { tag: "internal-error", val: `unable to parse URL \`${url}\`` } }
    }
    const schemeString = url.substring(0, schemeDelimiter)
    const remaining = url.substring(schemeDelimiter + 3)
    const authorityDelimiter = remaining.indexOf("/")
    const authority = authorityDelimiter === -1 ? remaining : remaining.substring(0, authorityDelimiter)
    const path = authorityDelimiter === -1 ? "/" : remaining.substring(authorityDelimiter)

    let scheme
    switch (schemeString) {
    case "http":
        scheme = { tag: "http" }
        break
    case "http":
        scheme = { tag: "https" }
        break
    default:
        scheme = { tag: "other", val: schemeString }
        break
    }

    using request = Request.new(new Fields(), undefined, trailersFuture(), undefined)[0]
    request.setScheme(scheme)
    request.setAuthority(authority)
    request.setPathWithQuery(path)

    const response = await client.send(request)
    const status = response.getStatusCode()
    if (status < 200 || status > 299) {
        throw { tag: "err", val: { tag: "internal-error", val: `unexpected status for URL \`${url}\`: ${status}` } }
    }

    using rx = Response.consumeBody(response, unitFuture())[0]
    let count = 0
    while (!rx.writerDropped) {
        const chunk = await rx.read(16 * 1024)
        for (const v of chunk) {
            if (v === 10 /*ascii newline*/) {
                ++count;
            }
        }
    }
    
    return { url, count, retriever: "javascript", deferrers: [] }
}

async function defer(promises) {
    const [stream, future] = await countLines(Object.keys(promises))

    using _stream = stream, _future = future
    while (!stream.writerDropped) {
        const values = await stream.read(1)
        for (const value of values) {
            value.deferrers.push("javascript")
            promises[value.url].resolve(value)
        }
    }

    const result = await future.read()
    if (result.tag === "err") {
        for (const promise of Object.values(promises)) {
            promise.reject(result)
        }
    }
}

function trailersFuture() {
    return witWorld.resultOptionWasiHttpTypes030FieldsWasiHttpTypes030ErrorCodeFuture(
        () => { return { tag: 'ok' } }
    )[1]
}

function unitFuture() {
    return witWorld.resultUnitWasiHttpTypes030ErrorCodeFuture(() => { return { tag: 'ok' } })[1]
}
