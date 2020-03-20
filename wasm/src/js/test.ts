import wasm_bin from '@text-render-rust/wasm/typesetting_bg.wasm'
import wasmInit, {Executor} from '@text-render-rust/wasm'

let initFlag = false
let executor: Executor
const init = async () => {
  await wasmInit(wasm_bin)
  executor = new Executor()
  initFlag = true
}

const test_text = {
  "width": 400.1,
  "height": 400,
  "paragraph": {
    "textAlign": "justify",
    "resizing": "grow-vertically",
    "align": "middle",
    "paragraphSpacing": 0,
    "advancedData": {
      "repeat": []
    },
    "shadow": null,
    "contents": [
      {
        "lineHeight": 1.2,
        "paragraphIndentation": 0,
        "blocks": [
          {
            "text": Date.now().toString(16).repeat(20),
            "fontFamily": "default",
            "fontSize": 36,
            "letterSpacing": 0,
            "fill": "#ff00cc",
            "italic": false,
            "stroke": "#000000",
            "strokeWidth": 0,
            "decoration": "underline"
          }
        ]
      }
    ]
  }
}

import default_font from 'file-loader!./c_739.ttf'


const test = async () => {
  if (!initFlag) await init()
  let time = performance.now()
  // @ts-ignore
  let arrayBuffer = await fetch(default_font).then(res => res.arrayBuffer())
  executor.loadFontBuffer('default', new Uint8Array(arrayBuffer))
  let commands = executor.exec(JSON.stringify(test_text))
  console.log('executor', performance.now() - time, commands)
}
// @ts-ignore
window.test = test