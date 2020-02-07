import wasm_bin from '@text-render-rust/wasm/wasm_bg.wasm'
import wasmInit, {Executor} from '@text-render-rust/wasm'

let initFlag = false
let executor: Executor
const init = async () => {
    await wasmInit(wasm_bin)
    executor = new Executor()
    initFlag = true
    executor.loadFont('default')
}

// @ts-ignore
import test_text from './t.js'
// @ts-ignore
import test_font from './f.js'

const test = async () => {
    if (!initFlag) await init()
    let time = performance.now()
    let commands = executor.exec(JSON.stringify(test_text))
    console.log('executor', performance.now() - time)
}

// @ts-ignore
window.getFontData = fontFamily => {
    console.log('getFontData')
    return JSON.stringify(test_font)
}
// @ts-ignore
window.test = test