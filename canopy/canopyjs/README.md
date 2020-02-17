<!-- Copyright 2018-2020 Cargill Incorporated

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License. -->

# CanopyJS

CanopyJS is a library for building Canopy applications. A Canopy application is
a React app that is capable of dynamically loading in saplings, which are UI
components designed to work with Splinter.

The central component provided by CanopyJS is a React context provider called
`CanopyProvider`. This context provider should wrap the top level component of
a Canopy application.

## Features

- Provides functionality for loading saplings into the Canopy application
- Implements some of the functions that are defined in SaplingJS
- Exposes shared configuration to saplings and Canopy application components

## Configuration

CanopyJS makes use of two endpoints, `splinterURL` and `saplingURL`.

### splinterURL

`splinterURL` is the URL where the Splinter daemon is running. This URL will be
used by Canopy and saplings to interact with Splinter via the Splinter daemon's
REST API. Examples of these interactions would include:
- Submitting transactions to a Scabbard service
- Managing users using the Biome module of Splinter

### saplingURL

`saplingURL` is the URL where saplings are being served from. On startup,
canopyJS will attempt to fetch sapling configuration from the following
endpoints:

- `${saplingURL}/configSaplings`: Config saplings
- `${saplingURL}/userSaplings`: User saplings

See the example in `splinter/canopy/app/saplings` for an example of these
configuration responses.

## Example

### App.js

```javascript
import React from 'react';
import { CanopyProvider } from 'canopyjs';

import SideNav from './components/SideNav';

function CanopyApp() {
  return (
    <CanopyProvider
      saplingURL={process.env.REACT_APP_SAPLING_URL}
      splinterURL={process.env.REACT_APP_SPLINTER_URL}
    >
      <SideNav />
    </CanopyProvider>
  );
}
export default CanopyApp;
```

In this example, `saplingURL` and `splinterURL` are set as React app environment
variables prior to starting up the application. The `SideNav` component gets
wrapped by the `CanopyProvider`, which gives it access to the React context
provided by CanopyJS.

### SideNav.js

```javascript
import React from 'react';
import { useUserSaplings } from 'canopyjs';

import NavItem from './NavItem';

function SideNav() {
  const userSaplings = useUserSaplings();
  const userSaplingRoutes = userSaplings.map(
    ({ displayName, namespace, icon }) => {
      return {
        path: `/${namespace}`,
        displayName,
        logo: icon
      };
    }
  );
  const userSaplingTabs = userSaplingRoutes.map(
    ({ path, displayName, logo }) => {
      return <NavItem key={path} path={path} label={displayName} logo={logo} />;
    }
  );

  return (
    <>
      <a href="/">
        <h2>Canopy</h2>
      </a>
      <div>{userSaplingTabs}</div>
    </>
  );
}

export default SideNav;
```

The `SideNav` component imports `useUserSaplings` from CanopyJS. The
`useUserSaplings` function exposes the part of the Canopy context that contains
user sapling configuration. This allows the `SideNav` component to render
`NavItems` for each of the user saplings. CanopyJS handles mounting the styles
and DOM elements for the currently active sapling.
