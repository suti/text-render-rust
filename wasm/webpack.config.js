const path = require('path')

function resolve(dir) {
    return path.join(__dirname, './', dir)
}

const devConfig = {
    entry: [resolve('./src/js/index.ts')],
    output: {
        publicPath: '/',
        filename: 'build.js',
        globalObject: 'this'
    },
    mode: 'development',
    module: {
        rules: [
            {
                test: /\.js$/,
                use: {
                    loader: 'babel-loader',
                    options: {
                        "plugins": [
                            "@babel/plugin-transform-runtime",
                            "@babel/plugin-syntax-import-meta"
                        ]
                    }
                },
                // exclude: /node_modules/
            },
            {
                test: /wasm\.js$/,
                loader: require.resolve('@open-wc/webpack-import-meta-loader')
                // exclude: /node_modules/
            },
            {
                test: /\.ts$/,
                use: {
                    loader: 'ts-loader',
                    options: {},
                },
            },
            {
                test: /typesetting\.js$/,
                loader: require.resolve('@open-wc/webpack-import-meta-loader'),
            },
            {
                test: /\.wasm$/,
                type: 'javascript/auto', /** this disabled webpacks default handling of wasm */
                use: [
                    {
                        loader: 'file-loader',
                        options: {
                            name: 'wasm/[name].[hash].[ext]',
                            publicPath: '../'
                        }
                    }
                ]
            }
        ]
    },
    devServer: {
        historyApiFallback: true,
        noInfo: true,
        // public: 'local.chuangkit.com',
        hot: true,
        inline: true,
        host: '0.0.0.0',
        port: 233,
        progress: true,
    },
    performance: {
        hints: false
    },
    devtool: '#cheap-module-eval-source-map'
}

module.exports = devConfig
