const path = require('path');
const nodeExternals = require('webpack-node-externals');
const CopyPlugin = require('copy-webpack-plugin'); // 1. 引入插件

module.exports = {
  // Set the mode to production for optimizations
  mode: 'production',

  // Define the entry point of your application
  entry: './index.js', // Adjust this to your main server file

  // Define the output of the bundle
  output: {
    path: path.resolve(__dirname, 'build'),
    filename: 'bundle.js'
  },

  node:{
        __dirname: false,
  },
  // Target a Node.js environment
  target: 'node',

  // Exclude node_modules from the bundle
  externals: [nodeExternals()],

  // Define the rules for processing different file types
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader'
        }
      }
    ]
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        // 复制 views 文件夹到 build/views
        { from: './views', to: 'views' },
        // 如果你还有 public 文件夹，也一并复制
        { from: './public', to: 'public' }
      ],
    }),
  ],
};