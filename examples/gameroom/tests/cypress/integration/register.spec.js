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


context('Unauthorized', function() {
  it('Failed access', function() {
    cy.log('Visiting the dashboard, unauthed')
    cy.visit('/index.html#/dashboard/home')
    cy.url().should('include', 'login')
  })
})

context('Register', function() {
  let aliceEmail
  let privateKey
  let password

  beforeEach(function() {
    cy.exec('npm run-script clear:db')
    cy.fixture('views.json').then(views => {
      cy.visit(views.register)
    })

    cy.fixture('credentials.json').then(user => {
      aliceEmail = user.aliceEmail
      privateKey = user.privateKey
      password = user.password
    })

  })

  it('Validate generatePrivateKey button', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.dataCy('generatePrivateKey').click()
    cy.dataCy('privateKey').invoke('val').then(value => {
      expect(value).to.have.lengthOf(64)
      expect(value).to.match(/^[a-zA-Z0-9]+$/)
    })

    cy.dataCy('submit').should('be.disabled')
  })

  it('Happy path register with generated key', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.register(aliceEmail, '1234', password, password)

    cy.dataCy('submit').click()
    cy.url().should('contain', 'register')

    cy.dataCy('generatePrivateKey').click()
    cy.dataCy('submit').click()
    cy.url().should('contain', 'dashboard')
  })

  it('Different Password', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.register(aliceEmail, privateKey,
                password, 'different_password')
    cy.dataCy('submit').click()

    cy.url().should('contain', 'register')
  })

  it('Bad Public Key', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.register(aliceEmail, '1234', password, password)
    cy.dataCy('submit').click()

    cy.url().should('contain', 'register')

  })

  it('Happy path register', function() {
    cy.dataCy('submit').should('be.disabled')
    cy.register(aliceEmail, privateKey, password, password)
    cy.dataCy('submit').click()

    cy.url().should('contain', 'dashboard')
  })
})
