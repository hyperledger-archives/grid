// Copyright 2018-2020 Cargill Incorporated
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

context('Login', function() {
  let aliceEmail
  let bobEmail
  let privateKey
  let password

  beforeEach(function() {
    cy.exec('npm run-script clear:db')
    cy.fixture('credentials.json').then(credentials => {
      aliceEmail = credentials.aliceEmail
      bobEmail = credentials.bobEmail
      privateKey = credentials.privateKey
      password = credentials.password
    })
    cy.fixture('views.json').then(views => {
      cy.visit(views.register)
      cy.register(aliceEmail, privateKey, password, password)
      cy.dataCy('submit').click()
      cy.url().should('contain', 'dashboard')
      cy.visit(views.login)
    })
  })

  it('Unregistered User', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.login(bobEmail, password)
    cy.dataCy('submit').click()
    cy.url().should('contain', 'login')
  })

  it('Wrong Password', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.login(aliceEmail, '1234')
    cy.dataCy('submit').click()
    cy.url().should('contain', 'login')
  })

  it('Happy Path login', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.login(aliceEmail, password)
    cy.dataCy('submit').click()
    cy.url().should('contain', 'dashboard')
  })
})
