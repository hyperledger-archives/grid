/**
 * Copyright 2018-2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const svgToMiniDataURI = require('mini-svg-data-uri');

module.exports = (env, argv) => {
  const isDevelMode = argv.mode === 'development';

  const config = {
    entry: [`${__dirname}/src/index.js`],
    output: {
      filename: 'profile.js',
      path: path.resolve(__dirname, 'build/static/js')
    },
    resolve: {
      alias: {
        App: path.resolve(__dirname, 'src')
      }
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
            name: 'profile',
            chunks: 'all',
            minChunks: 2
          }
        }
      }
    },
    plugins: [
      new MiniCssExtractPlugin({
        // this file path is relative to the js file output
        filename: '../css/profile.css'
      })
    ]
  };

  if (isDevelMode) {
    config.devtool = 'source-map';
  }

  return config;
};
