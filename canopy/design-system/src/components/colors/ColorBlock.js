/**
 * Copyright 2019 Cargill Incorporated
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
import React from 'react';

export function ColorBlock({ colors }) {
  return (
    <div  className="color-block display-flex flexDirection-column alignItems-center"
          style={{width: '100%', borderRadius: '4px', height: '10rem'}}>
      <div className="colors display-flex flexDirection-column"
            style={{width: '100%', height: '100%'}}>
        {colors.map(color => {
          return (
            <div
              className={`color background-${color} display-flex justifyContent-center alignItems-center`}
              style={{width: '100%'}}
              key={color.name}
            >
              {color}
            </div>
          );
        })}
      </div>
    </div>
  );
}
