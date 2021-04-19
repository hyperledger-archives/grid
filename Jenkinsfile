#!groovy

// Copyright 2017 Intel Corporation
// Copyright 2020 Cargill Incorporated
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
// ------------------------------------------------------------------------------

pipeline {
    agent {
        node {
            label 'master'
            customWorkspace "workspace/${env.BUILD_TAG}"
        }
    }

    triggers {
        cron(env.BRANCH_NAME == 'main' ? 'H 2 * * *' : '')
    }

    options {
        timestamps()
        buildDiscarder(logRotator(daysToKeepStr: '31'))
    }

    environment {
        ISOLATION_ID = sh(returnStdout: true, script: 'printf $BUILD_TAG | sha256sum | cut -c1-64').trim()
        COMPOSE_PROJECT_NAME = sh(returnStdout: true, script: 'printf $BUILD_TAG | sha256sum | cut -c1-64').trim()
        JENKINS_UID = sh(returnStdout: true, script: "id -u ${USER}").trim()
        VERSION = "AUTO_STRICT"
    }

    stages {
        stage('Check Whitelist') {
            steps {
                readTrusted 'bin/whitelist'
                sh './bin/whitelist "$CHANGE_AUTHOR" /etc/jenkins-authorized-builders'
            }
            when {
                not {
                    branch 'main'
                }
            }
        }

        stage("Run Lint on Grid UI") {
            steps {
                sh 'just ci-lint-ui'
            }
        }

        stage("Run Grid UI tests") {
            steps {
                sh 'just ci-test-ui'
            }
        }

        stage("Run Lint on Grid") {
            steps {
                sh 'just ci-lint-grid'
            }
        }
        stage("Run Grid unit tests") {
            steps {
                sh 'just ci-unit-test-grid'
            }
        }

        stage("Run integration tests") {
            steps {
                sh 'just integration-test'
            }
        }

        stage("Build gridd with experimental features") {
            steps {
                sh 'ISOLATION_ID=$ISOLATION_ID"experimental" CARGO_ARGS=" --features experimental" docker-compose -f docker-compose.yaml build gridd'
            }
        }

        stage("Create git archive") {
            steps {
                sh '''
                    REPO=$(git remote show -n origin | grep Fetch | awk -F'[/.]' '{print $6}')
                    VERSION=`git describe --dirty`
                    git archive HEAD --format=zip -9 --output=$REPO-$VERSION.zip
                    git archive HEAD --format=tgz -9 --output=$REPO-$VERSION.tgz
                '''
            }
        }

        stage("Build artifacts") {
            steps {
                sh 'mkdir -p build/debs'
                sh 'docker-compose -f docker/compose/copy-artifacts.yaml up'
            }
        }
    }

    post {
        always {
            sh 'docker-compose -f docker/compose/grid_tests.yaml down'
            sh 'docker-compose -f docker/compose/copy-artifacts.yaml down'
        }
        success {
            archiveArtifacts '*.tgz, *.zip, build/debs/*.deb, build/scar/*.scar'
        }
        aborted {
            error "Aborted, exiting now"
        }
        failure {
            error "Failed, exiting now"
        }
    }
}
