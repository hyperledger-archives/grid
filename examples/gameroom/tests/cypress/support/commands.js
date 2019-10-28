// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


Cypress.Commands.add("dataCy", (value) => cy.get(`[data-cy=${value}]`))

Cypress.Commands.add("register", (email, privateKey, password, confirmPassword) => {
  cy.dataCy('email').clear().type(email)
  cy.dataCy('privateKey').clear().type(privateKey)
  cy.dataCy('password').clear().type(password)
  cy.dataCy('confirmPassword').clear().type(confirmPassword)
})

Cypress.Commands.add("login", (email, password) => {
  cy.dataCy('email').clear().type(email)
  cy.dataCy('password').clear().type(password)
})
