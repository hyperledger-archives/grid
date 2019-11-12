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
interface RegisterApp {
  (bootstrapFunction: (domNode: Node) => void): void;
}

interface RegisterConfigSapling {
  (
    configNamespace: 'login' | 'notifications',
    bootstrapFunction: () => void
  ): void;
}
interface Canopy {
  registerApp: RegisterApp;
  registerConfigSapling: RegisterConfigSapling;
}

function assertAndGetWindowCanopy(): Canopy {
  // In order to prevent the need to overwrite the window interface,
  // a intentional `any` is cast here.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if (!window || !(window as any).$CANOPY) {
    throw new Error(
      `Must be in a Canopy with 'window.$CANOPY' in scope to call this CanopyJS functions`
    );
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (window as any).$CANOPY;
}

const canopy = assertAndGetWindowCanopy();

export const { registerApp, registerConfigSapling }: Canopy = canopy;
