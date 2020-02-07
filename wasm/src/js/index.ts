import Worker from 'worker-loader!./worker.ts'

class TextRender {
  private readonly worker!: Worker
  private coreLoadBack!: () => void
  private preloadFontResolve: { [ff: string]: (arg?: PromiseLike<void>) => void } = {}
  private coreStatus: Promise<any> = new Promise<void>(((resolve, reject) => {
    this.coreLoadBack = resolve
  }))
  private coreRenderBack!: (commands: any) => void
  public getFontData: (ff: string) => Promise<string> = s => Promise.resolve(s)
  public fallBack!: () => any

  constructor (cb?: () => any) {
    if (cb)
      this.fallBack = cb
    this.worker = new Worker()
    this.worker.onmessage = (message: MessageEvent) => {
      let {data: {type, content: {commands, fontFamily} = {commands: '', fontFamily: ''}, content}} = message
      switch (type) {
        case 'requestCache':
          this.getFontData(fontFamily).then(cache => {
            this.worker.postMessage({
              type: 'fontCache',
              content: {cache, fontFamily},
            })
          })
          break
        case 'loaded':
          this.coreLoadBack()
          break
        case 'ok':
          this.coreRenderBack(content)
          break
        case 'err':
          console.error(content)
          break
        case 'loadErr':
          console.error(content)
          this.fallBack && this.fallBack()
          break
        case 'loadedFont':
          if (this.preloadFontResolve[fontFamily]) {
            this.preloadFontResolve[fontFamily]()
            delete this.preloadFontResolve[fontFamily]
          }
          break
        default:

      }
    }
  }

  private _exec (data: any): Promise<string> {
    return new Promise(resolve => {
      this.coreRenderBack = resolve
      this.worker.postMessage({type: 'textData', content: {textData: data}})
    })
  }

  bind (cb: (ff: string) => Promise<any>) {
    this.getFontData = cb
  }

  exec (data: any): Promise<any> {
    return new Promise(resolve => {
      this.coreStatus = this.coreStatus.then(async () => {
        let result = await this._exec(data)
        resolve(result)
      })
    })
  }

  preload (ff: string): Promise<void> {
    this.worker.postMessage({type: 'preload', content: {fontFamily: ff}})
    return new Promise(resolve => {
      this.preloadFontResolve[ff] = resolve
    })
  }

  terminate () {
    this.worker.terminate()
  }
}

export default TextRender

export function draw(ctx: CanvasRenderingContext2D, commands: any[]) {
  commands.forEach(command => {
    const {type, value, width} = command
    switch (type) {
      case 'path':
        ctx.beginPath()
        value.forEach((cmd: { type: string, x: number, y: number, x1: number, y1: number, x2: number, y2: number }) => {
          const {type, x, y, x1, y1, x2, y2} = cmd
          switch (type) {
            case 'move':
              ctx.moveTo(x, y)
              break
            case 'line':
              ctx.lineTo(x, y)
              break
            case 'curve':
              ctx.bezierCurveTo(x1, y1, x2, y2, x, y)
              break
            case 'close':
              ctx.closePath()
              break
          }
        })
        break
      case 'fill':
        ctx.fillStyle = value
        ctx.fill()
        break
      case 'stroke':
        ctx.strokeStyle = value
        ctx.lineWidth = width
        ctx.stroke()
        break
      case 'transform':
        let {a, b, c, d, e, f} = value
        ctx.setTransform(a, b, c, d, e, f)
        break
    }
  })
}

// @ts-ignore
import test_text from './t.js'
// @ts-ignore
import test_font from './f.js'

let textRender = new TextRender()
textRender.bind(async () => JSON.stringify(test_font))
let canvas = document.createElement('canvas')
canvas.width = test_text().width
canvas.height = test_text().height
let ctx = canvas.getContext('2d') as CanvasRenderingContext2D
document.body.append(canvas)

export async function test() {
  let j = test_text()
  console.time('time')
  let result = await textRender.exec(j)
  console.timeEnd('time')
  // @ts-ignore
  window.result = result
  // let {box, commands} = JSON.parse(result)
  // draw(ctx, result.commands)
  // console.log(performance.now() - time)

}

// @ts-ignore
window.test = test