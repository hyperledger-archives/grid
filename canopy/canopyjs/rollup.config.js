/**
 * Copyright 2018-2020 Cargill Incorporated
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
import sass from 'rollup-plugin-sass';
import babel from 'rollup-plugin-babel';
import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import external from 'rollup-plugin-peer-deps-external';
import analyzer from 'rollup-plugin-analyzer';
import { terser } from 'rollup-plugin-terser';
import { uglify } from 'rollup-plugin-uglify';
import autoprefixer from 'autoprefixer';
import postcss from 'postcss';
import base64 from 'postcss-base64';
import clean from 'postcss-clean';
import fs from 'fs';
import packageJSON from './package.json';

const themes = fs.readdirSync('./src/themes');
const components = './src/index.js';
const minifyExtension = pathToFile => pathToFile.replace(/\.js$/, '.min.js');

const opts = {
  extensions: ['.png', '.svg']
};

const themeBundles = themes.map(theme => {
  return {
    input: `./src/themes/${theme}/index.js`,
    output: {
      file: `build/themes/${theme}/index.js`,
      format: 'esm'
    },
    plugins: [
      sass({
        output: true,
        processor: css =>
          postcss([autoprefixer, base64(opts), clean()])
            .process(css)
            .then(result => result.css)
      })
    ]
  };
});

export default [
  // style themes
  ...themeBundles,
  // commonjs
  {
    input: components,
    output: {
      file: packageJSON.components,
      format: 'cjs',
      sourcemap: true
    },
    plugins: [
      babel({
        exclude: '/node_modules/**',
        runtimeHelpers: true
      }),
      external(),
      resolve(),
      commonjs({
        namedExports: {
          'react-is': ['isValidElementType']
        }
      }),
      analyzer()
    ]
  },
  {
    input: components,
    output: {
      file: minifyExtension(packageJSON.components),
      format: 'cjs',
      sourcemap: true
    },
    plugins: [
      babel({
        exclude: 'node_modules/**',
        runtimeHelpers: true
      }),
      external(),
      resolve(),
      commonjs({
        namedExports: {
          'react-is': ['isValidElementType']
        }
      }),
      uglify(),
      analyzer()
    ]
  }
];
