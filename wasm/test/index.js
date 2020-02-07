console.log('onload')
import t from '../src/js/t.js'
import f from '../src/js/f.js'

window.getFontData = (a) => {
    console.log(a, f)
    return Promise.resolve(JSON.stringify(f))
}

import('../pkg/wasm.js').then(wasm => {
    // let e = new wasm.Executor()
    // let td = JSON.stringify(t)
    // console.log(td)
    // e.exec(td)
    let r = wasm.s_test("nihao")
    console.log(r)
})

