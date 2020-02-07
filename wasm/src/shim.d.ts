declare module 'file-loader!*' {
    export var bin: string
}

declare module '*.wasm' {
    export var bin: string
}

declare module 'worker-loader!*' {
    class WebpackWorker extends Worker {
        constructor()
    }

    export default WebpackWorker
}