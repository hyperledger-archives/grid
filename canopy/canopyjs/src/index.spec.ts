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

const bootstrap = (): void => {
  /* no op */
};
const completeUser = { userId: 'COMPLETE', displayName: 'canopy' };
const minimalUser = { userId: 'MINIMAL' };

// In order to prevent the need to overwrite the window interface,
// a intentional `any` is cast here.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(window as any).$CANOPY = {
  registerConfigSapling: jest.fn(),
  registerApp: jest.fn(),
  getUser: jest.fn(() => completeUser),
  setUser: jest.fn(),
  getSharedConfig: jest.fn(() => ({
    mock: true
  }))
};

interface MockCanopy {
  registerConfigSapling: jest.Mock;
  registerApp: jest.Mock;
  getUser: jest.Mock;
  setUser: jest.Mock;
  getSharedConfig: jest.Mock;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const $CANOPY = (window as any).$CANOPY as MockCanopy;

describe('CanopyJS', () => {
  afterEach(() => {
    jest.clearAllMocks();
  });
  describe('registerApp(bootstrapFn)', () => {
    it('should call the window.$CANOPY.registerApp function with the same signature as the register function', async () => {
      expect.assertions(1);
      // dynamic import is used here to ensure that the window.$CANOPY object has been set up
      const { registerApp } = await import('./index');
      registerApp(bootstrap);
      expect($CANOPY.registerApp.mock.calls[0][0]).toEqual(bootstrap);
    });
  });

  describe('registerConfigSapling(configNamespace, bootstrapFn)', () => {
    it('should register to the window Canopy object', async () => {
      expect.assertions(2);
      // dynamic import is used here to ensure that the window.$CANOPY object has been set up
      const { registerConfigSapling } = await import('./index');
      registerConfigSapling('login', bootstrap);
      expect($CANOPY.registerConfigSapling.mock.calls[0][0]).toEqual('login');
      expect($CANOPY.registerConfigSapling.mock.calls[0][1]).toEqual(bootstrap);
    });
  });

  describe('getUser()', () => {
    it('should call getUser from window object', async () => {
      expect.assertions(1);
      // dynamic import is used here to ensure that the window.$CANOPY object has been set up
      const { getUser } = await import('./index');
      expect(getUser()).toEqual(completeUser);
    });
  });

  describe('setUser(user)', () => {
    it('should call setUser from window object with a complete user object', async () => {
      expect.assertions(1);
      // dynamic import is used here to ensure that the window.$CANOPY object has been set up
      const { setUser } = await import('./index');
      setUser(completeUser);
      expect($CANOPY.setUser.mock.calls[0][0]).toEqual(completeUser);
    });
    it('it should call setUser from window object with a minimal user object', async () => {
      expect.assertions(1);
      // dynamic import is used here to ensure that the window.$CANOPY object has been set up
      const { setUser } = await import('./index');
      setUser(minimalUser);
      expect($CANOPY.setUser.mock.calls[0][0]).toEqual(minimalUser);
    });
  });

  describe('getSharedConfig', () => {
    it('should call getSharedConfig from window object', async () => {
      const { getSharedConfig } = await import('./index');
      expect(getSharedConfig()).toEqual({ mock: true });
    });
  });
});
