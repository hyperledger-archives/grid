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
import React, { lazy, Suspense } from 'react';
import {
  BrowserRouter as Router,
  Route,
  Switch,
  Redirect
} from 'react-router-dom';

import './App.scss';

import SideNav from './components/navigation/SideNav';

const tabs = [
  {
    name: 'Overview',
    nested: [
      {
        name: 'Introduction',
        route: '/overview/introduction',
        component: lazy(() =>
          import('!babel-loader!mdx-loader!./views/Introduction.mdx')
        )
      },
      {
        name: 'Conventions',
        route: '/overview/conventions'
      }
    ]
  },
  {
    name: 'Design',
    nested: [
      {
        name: 'Colors',
        route: '/design/colors',
        component: lazy(() =>
          import('!babel-loader!mdx-loader!./views/Colors.mdx')
        )
      },
      {
        name: 'Buttons',
        route: '/design/buttons'
      },
      {
        name: 'Typography',
        route: '/design/typography',
        component: lazy(() =>
          import('!babel-loader!mdx-loader!./views/Typography.mdx')
        )
      }
    ]
  },
  {
    name: 'Components',
    nested: [
      {
        name: 'Progress',
        route: '/components/progress',
        component: lazy(() =>
          import('!babel-loader!mdx-loader!./views/Progress.mdx')
        )
      },
      {
        name: 'TabBox',
        route: '/components/tabbox',
        component: lazy(() =>
          import('!babel-loader!mdx-loader!./views/TabBox.mdx')
        )
      }
    ]
  }
];

const flatRoutes = tabs
  .map(t => t.nested)
  .flat()
  .filter(({ component }) => Boolean(component));

function App() {
  return (
    <Router>
      <div className="App">
        <SideNav tabs={tabs} />
        <div className="view marginLeft-l marginRight-l paddingTop-l">
          <Switch>
            <Redirect exact from="/" to="/overview/introduction" />
            <Redirect exact from="/overview" to="/overview/introduction" />
            <Redirect exact from="/design" to="/design/colors" />
            {flatRoutes.map(({ route, component }) => {
              const C = component;
              return (
                <Route path={route} exact key={route}>
                  <Suspense fallback={<div>Loading...</div>}>
                    <C />
                  </Suspense>
                </Route>
              );
            })}
          </Switch>
        </div>
      </div>
    </Router>
  );
}

export default App;
