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

import { register } from './register';
import { initialize } from './initialize';

const bootstrap = (): void => {
  /* no op */
};

describe('Register', () => {
  beforeEach(() => {
    initialize();
  });
  afterEach(() => {
    delete window.$CANOPY;
  });
  it('should register a bootstrap function', () => {
    register(bootstrap);
    expect(window.$CANOPY.invokeRegisteredApp).toEqual(bootstrap);
  });

  it('should throw if register if called more than once', () => {
    register(bootstrap);
    expect(() => register(bootstrap)).toThrow();
  });

  it('ignore the argument of a second call to register', () => {
    expect.assertions(2);
    const secondBootstrap = (): void => {
      /* no op */
    };
    register(bootstrap);
    expect(() => register(secondBootstrap)).toThrow();
    expect(window.$CANOPY.invokeRegisteredApp).not.toEqual(secondBootstrap);
  });
});
