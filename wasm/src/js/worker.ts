import wasm_bin from '@text-render-rust/wasm/typesetting_bg.wasm'
import wasmInit, {Executor} from '@text-render-rust/wasm'

declare interface WorkerSelf extends Worker {
  getFontData: (fontFamily: string) => string
  getDate: () => number
}

let initFlag = false
let loadedFontName: string[] = []
let preparedFontCaches: { [name: string]: string } = {}
let executor: Executor
const SCOPE: WorkerSelf = self as any
const init = async () => {
  await wasmInit(wasm_bin)
  executor = new Executor()
  initFlag = true
}
const loadFont = (fontFamily: string) => new Promise<string>(resolve => {
  if (loadedFontName.includes(fontFamily)) return resolve()
  const onMessage = (message: MessageEvent) => {
    let {data: {type, content}} = message
    if (content === undefined) return
    const {cache, fontFamily: ff} = content
    if (type === 'fontCache' && fontFamily === ff) {
      SCOPE.removeEventListener('message', onMessage)
      preparedFontCaches[fontFamily] = cache
      resolve(cache)
    }
  }
  SCOPE.addEventListener('message', onMessage)
  SCOPE.postMessage({type: 'requestCache', content: {fontFamily}})
})

const loadFontAll = async (textData: TextData) => {
  let fontFamilyList: string[] = []
  textData.paragraph.contents.forEach(content => {
    content.blocks.map(block => {
      fontFamilyList.push(block.fontFamily)
    })
  })
  fontFamilyList = [...new Set(fontFamilyList)]
  await Promise.all(fontFamilyList.map(ff => loadFont(ff)))
}

SCOPE.getFontData = fontFamily => {
  let result = preparedFontCaches[fontFamily]
  loadedFontName.push(fontFamily)
  delete preparedFontCaches[fontFamily]
  return result
}

SCOPE.getDate = () => self.performance.now()

const onMessage = async (message: MessageEvent) => {
  let {data: {type, content}} = message
  if (content === undefined) return
  let {textData, fontFamily} = content
  if (!initFlag) return console.warn('text worker have not loaded!')
  switch (type) {
    case 'preload':
      await loadFont(fontFamily)
      executor.loadFont(fontFamily)
      SCOPE.postMessage({type: 'loadedFont', content: {fontFamily}})
      break
    case 'textData':
      await loadFontAll(textData)
      textData.paragraph.advancedData = {repeat: []}
      textData.paragraph.shadow = null
      try {
        console.time('rust exec')
        let result = executor.exec(JSON.stringify(textData))
        console.timeEnd('rust exec')
        console.time('box compute')
        let boxLen = result[0] * 4
        let boxes = new Array(result[0]).fill(undefined).map((_, i) => {
          let from = i * 4 + 1
          let x1 = result[from++]
          let y1 = result[from++]
          let x2 = result[from++]
          let y2 = result[from++]
          return {x1, y1, x2, y2}
        })
        console.timeEnd('box compute')
        console.time('command compute')
        let commands = transferArray(result.slice(boxLen + 1))
        console.timeEnd('command compute')
        SCOPE.postMessage({type: 'ok', content: {boxes, commands}})
      } catch (e) {
        console.error(e)
        SCOPE.postMessage({type: 'err', content: {message: e.toString(), textData: JSON.stringify(textData)}})
      }
      break
    case 'requestCache':
    case 'fontCache':
      break
    default:
      console.warn('type::', type)
  }
}

SCOPE.addEventListener('message', onMessage)

init().then(async () => {
  console.info('text worker load success!')
  await loadFont('default')
  executor.loadFont('default')
  SCOPE.postMessage({type: 'loaded'})
}).catch(e => {
  SCOPE.postMessage({type: 'loadErr', content: e})
})

interface TextData {
  width: number
  height: number
  paragraph: {
    textAlign: 'left' | 'center' | 'right' | 'justify'
    resizing: 'grow-horizontally' | 'grow-vertically' | 'fixed'
    align: 'top' | 'middle' | 'bottom'
    paragraphSpacing: number
    shadow: {
      blur: number
      offset: [number, number]
      color: string
    } | void
    advancedData: {
      repeat: {
        fill: string[]
        stroke: string
        strokeWidth: string
        shadow: {
          blur: number
          offset: [number, number]
          color: string
        } | void
      }[]
    }
    contents: {
      lineHeight: number
      paragraphIndentation: number
      blocks: {
        text: string
        fontFamily: string
        fontSize: number
        letterSpacing: number
        fill: string
        italic: boolean
        stroke: string
        strokeWidth: number
        decoration: 'underline' | 'overline' | 'line-through' | ''

      }[]
    }[]
  }
}

export function transferArray(arr: Float32Array) {
  const commandType = ['transform', 'path', 'stroke', 'fill']
  const pathType = ['move', 'line', 'curve', 'close', 'quad']
  const result: { type: string, value: any, [key: string]: any }[] = []
  let commandPoint = 0
  // console.log(arr.length)
  for (let i = 1; i < arr.length;) {
    let type = commandType[arr[i++]]
    let value = undefined
    let width = undefined
    if (type === undefined) {
      console.log('parse command array error! typedArray: ', arr, ',point: ', i - 1, ',commandPoint: ', commandPoint)
      throw new Error('parse command array error!')
    }
    switch (type) {
      case 'transform':
        let a = arr[i++]
        let b = arr[i++]
        let c = arr[i++]
        let d = arr[i++]
        let e = arr[i++]
        let f = arr[i++]
        value = {a, b, c, d, e, f}
        break
      case 'path':
        let pathCount = arr[i++]
        value = []
        while (pathCount > 0) {
          let type = pathType[arr[i++]]
          let x, y, x1, y1, x2, y2
          switch (type) {
            case 'move':
              x = arr[i++]
              y = arr[i++]
              break
            case 'line':
              x = arr[i++]
              y = arr[i++]
              break
            case 'curve':
              x = arr[i++]
              y = arr[i++]
              x1 = arr[i++]
              y1 = arr[i++]
              x2 = arr[i++]
              y2 = arr[i++]
              break
            case 'close':
              break
            case 'quad':
              x = arr[i++]
              y = arr[i++]
              x1 = arr[i++]
              y1 = arr[i++]
              break
            default:
              console.log('parse path command array error! typedArray: ', arr, ',point: ', i - 1, ',commandPoint: ', commandPoint)
              throw new Error('parse command array error!')
          }
          value.push({type, x, y, x1, y1, x2, y2})
          pathCount--
        }
        break
      case 'stroke':
        width = arr[i++]
        value = toHex(arr[i++])
        break
      case 'fill':
        value = toHex(arr[i++])
        break
    }
    result.push({type, value, width})
    commandPoint++
  }
  return result
}

function toHex(input: number) {
  let r = input >> 16
  let g = (input & 0x00FF00) >> 8
  let b = input & 0x0000FF
  let hex = [
    r.toString(16),
    g.toString(16),
    b.toString(16),
  ]
  Array.prototype.map.call(hex, function (val, nr) {
    if (val.length == 1) {
      hex[nr] = '0' + val
    }
  })
  return '#' + hex.join('')
}