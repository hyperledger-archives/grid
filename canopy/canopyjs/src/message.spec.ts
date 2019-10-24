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

import { message } from './message';

const goodMessage = {
  body: 'Canopy, the uppermost spreading branchy layer of a forest.',
  type: 'generic'
};

describe('Message', () => {
  describe('Arguments', () => {
    afterEach(() => {
      jest.clearAllMocks();
    });
    beforeEach(() => {
      jest.clearAllMocks();
    });
    it('Should pass the first argument a body to the message', () => {
      expect.assertions(2);

      const spy = jest.spyOn(window.parent, 'postMessage');
      message(goodMessage);

      expect(spy).toHaveBeenCalled();
      expect(spy.mock.calls[0][0].message).toEqual(goodMessage);
    });
  });

  describe('Register/Deregister', () => {
    let eventListenerSpy = null;

    afterEach(() => {
      jest.clearAllMocks();
    });
    beforeEach(() => {
      eventListenerSpy = jest.spyOn(window, 'addEventListener');
    });

    it('should register an event listener once', () => {
      expect.assertions(3);
      message(goodMessage);
      expect(eventListenerSpy).toHaveBeenCalled();
      expect(eventListenerSpy.mock.calls[0][0]).toEqual('message');
      expect(eventListenerSpy.mock.calls.length).toEqual(1);
    });

    it('should deregister an event listener', () => {
      expect.assertions(2);

      // mock the event registration so the callback can be invoked
      const events = {
        message: jest.fn()
      };
      window.addEventListener = jest.fn((event, cb) => {
        events[event] = cb;
      });

      // collect the uuid from postMessage
      let lastId = null;
      window.parent.postMessage = jest.fn(({ id }) => {
        lastId = id;
      });

      // mock remove event before invoking the call
      // TS cannot understand the new type that has been assigned here, there is an ignore at the expect statement.
      window.removeEventListener = jest.fn();

      // invoke the call
      message(goodMessage);

      // invoke the mock event with the correct ID
      events.message({ id: lastId });

      expect(window.removeEventListener).toHaveBeenCalled();

      // eslint-disable-next-line
      // @ts-ignore
      expect(window.removeEventListener.mock.calls.length).toEqual(1);
    });
  });

  describe('Message > Status', () => {
    it("should reject when status is not 'OK'", done => {
      // mock the event registration so the callback can be invoked
      const events = {
        message: jest.fn()
      };
      window.addEventListener = jest.fn((event, cb) => {
        events[event] = cb;
      });

      // collect the uuid from postMessage
      let lastId = null;
      window.parent.postMessage = jest.fn(({ id }) => {
        lastId = id;
      });

      // invoke the call
      message(goodMessage)
        .then(() => done.fail('Did not handle bad status correctly'))
        .catch(() => done());

      // invoke the mock with a bad status
      events.message({ id: lastId, status: 'Not OK' });
    });

    it("should resolve when status is 'OK'", done => {
      // mock the event registration so the callback can be invoked
      const events = {
        message: jest.fn()
      };
      window.addEventListener = jest.fn((event, cb) => {
        events[event] = cb;
      });

      // collect the uuid from postMessage
      let lastId = null;
      window.parent.postMessage = jest.fn(({ id }) => {
        lastId = id;
      });

      // invoke the call
      message(goodMessage)
        .then(() => done())
        .catch(() => done.fail('Did not handle good status correctly'));

      // invoke the mock event with the correct ID
      events.message({ id: lastId, status: 'OK' });
    });
  });
});
