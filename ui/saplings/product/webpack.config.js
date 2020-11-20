/** Copyright 2020 Cargill Incorporated */

const path = require('path');

const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const svgToMiniDataURI = require('mini-svg-data-uri');

module.exports = (env, argv) => {
  const isDevelMode = argv.mode === 'development';

  const config = {
    entry: [`${__dirname}/src/index.js`],
    output: {
      filename: 'product.js',
      path: path.resolve(__dirname, 'build/static/js')
    },
    module: {
      rules: [
        // jsx, etc
        {
          test: /\.js$/,
          exclude: /node_modules/,
          loader: 'babel-loader'
        },
        {
          test: /\.css$/,
          use: [
            'style-loader',
            MiniCssExtractPlugin.loader,
            {
              loader: 'css-loader',
              options: {
                importLoaders: 2,
                sourceMap: isDevelMode
              }
            }
          ]
        },
        {
          // styles
          test: /\.(sass|scss)$/,
          use: [
            'style-loader',
            MiniCssExtractPlugin.loader,
            {
              loader: 'css-loader',
              options: {
                importLoaders: 2,
                sourceMap: isDevelMode
              }
            },
            {
              loader: require.resolve('resolve-url-loader'),
              options: {
                sourceMap: isDevelMode
              }
            },
            {
              loader: 'sass-loader',
              options: {
                sourceMap: isDevelMode
              }
            }
          ]
        },
        {
          test: /\.svg$/i,
          use: [
            {
              loader: 'url-loader',
              options: {
                generator: context => svgToMiniDataURI(context.toString())
              }
            }
          ]
        }
      ]
    },
    optimization: {
      splitChunks: {
        cacheGroups: {
          css: {
            test: /\.(css|sass|scss)$/,
            name: 'product',
            chunks: 'all',
            minChunks: 2
          }
        }
      }
    },
    plugins: [
      new MiniCssExtractPlugin({
        // this file path is relative to the js file output
        filename: '../css/product.css'
      })
    ]
  };

  if (isDevelMode) {
    config.devtool = 'source-map';
  }

  return config;
};
