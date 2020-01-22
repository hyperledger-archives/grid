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

import React, { useState } from 'react';
import classnames from 'classnames';
import PropTypes from 'prop-types';

export default function TabBox({ contents, keyExtractor }) {
  const titles = contents.map(({ title }) => title);
  const contentElements = contents.map(({ content }) => content);
  const keys = contents.map(keyExtractor);

  const [selectedTab, setSelectedTab] = useState(0);
  const Content =
    contentElements[selectedTab] &&
    typeof contentElements[selectedTab] === 'string'
      ? () => <>{contentElements[selectedTab]}</>
      : contentElements[selectedTab];

  return (
    <div className="tab-box">
      <div className="tab-box-options">
        {titles.map((title, index) => {
          const Title = title;
          const selected = index === selectedTab;
          return (
            <button
              role="tab"
              type="button"
              aria-selected={selected}
              tabIndex={selected ? 0 : -1}
              key={keys[index]}
              className={classnames('tab-box-option', selected && 'active')}
              onClick={() => setSelectedTab(index)}
            >
              {typeof Title === 'string' ? Title : <Title />}
            </button>
          );
        })}
      </div>
      <div className="tab-box-content">
        <Content />
      </div>
    </div>
  );
}

TabBox.defaultProps = {
  keyExtractor: (_, contentIndex) => contentIndex
};

TabBox.propTypes = {
  contents: PropTypes.arrayOf(
    PropTypes.shape({
      title: PropTypes.oneOfType([PropTypes.string, PropTypes.func]),
      content: PropTypes.oneOfType([PropTypes.string, PropTypes.func])
    })
  ).isRequired,
  keyExtractor: PropTypes.func
};
