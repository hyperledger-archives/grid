#!groovy

// Copyright 2017 Intel Corporation
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

// Discard old builds after 31 days
properties([[$class: 'BuildDiscarderProperty', strategy:
        [$class: 'LogRotator', artifactDaysToKeepStr: '',
        artifactNumToKeepStr: '', daysToKeepStr: '31', numToKeepStr: '']]]);

node ('master') {
    // Create a unique workspace so Jenkins doesn't reuse an existing one
    ws("workspace/${env.BUILD_TAG}") {
        stage("Clone Repo") {
            checkout scm
            sh 'git fetch --tag'
        }

        if (!(env.BRANCH_NAME == 'master' && env.JOB_BASE_NAME == 'master')) {
            stage("Check Whitelist") {
                readTrusted 'bin/whitelist'
                sh './bin/whitelist "$CHANGE_AUTHOR" /etc/jenkins-authorized-builders'
            }
        }

        stage("Check for Signed-Off Commits") {
            sh '''#!/bin/bash -l
                if [ -v CHANGE_URL ] ;
                then
                    temp_url="$(echo $CHANGE_URL |sed s#github.com/#api.github.com/repos/#)/commits"
                    pull_url="$(echo $temp_url |sed s#pull#pulls#)"

                    IFS=$'\n'
                    for m in $(curl -s "$pull_url" | grep "message") ; do
                        if echo "$m" | grep -qi signed-off-by:
                        then
                          continue
                        else
                          echo "FAIL: Missing Signed-Off Field"
                          echo "$m"
                          exit 1
                        fi
                    done
                    unset IFS;
                fi
            '''
        }

        // Set the ISOLATION_ID environment variable for the whole pipeline
        env.ISOLATION_ID = sh(returnStdout: true, script: 'printf $BUILD_TAG | sha256sum | cut -c1-64').trim()
        env.COMPOSE_PROJECT_NAME = sh(returnStdout: true, script: 'printf $BUILD_TAG | sha256sum | cut -c1-64').trim()
        env.JENKINS_UID = sh(returnStdout: true, script: "id -u ${USER}").trim()

        stage("Run Lint") {
            sh 'docker build . -f docker/lint -t lint-grid:$ISOLATION_ID'
            sh 'docker run --rm -v $(pwd):/project/grid lint-grid:$ISOLATION_ID'
        }

        // Use a docker container to build and protogen, so that the Jenkins
        // environment doesn't need all the dependencies.
        stage("Build Test Dependencies") {
            sh 'VERSION=AUTO_STRICT REPO_VERSION=$(./bin/get_version) docker-compose -f docker-compose-installed.yaml build --force-rm'
            sh 'docker-compose -f docker/compose/grid_tests.yaml build --force-rm'
        }

        stage("Run unit tests") {
            sh 'docker-compose -f docker/compose/grid_tests.yaml up --abort-on-container-exit --exit-code-from grid_tests'
            sh 'docker-compose -f docker/compose/grid_tests.yaml down'
        }

         stage("Run integration tests") {
            sh './bin/run_integration_tests'
         }

        stage("Create git archive") {
            sh '''
                REPO=$(git remote show -n origin | grep Fetch | awk -F'[/.]' '{print $6}')
                VERSION=`git describe --dirty`
                git archive HEAD --format=zip -9 --output=$REPO-$VERSION.zip
                git archive HEAD --format=tgz -9 --output=$REPO-$VERSION.tgz
            '''
        }

        stage ("Build documentation") {
            sh 'docker build . -f docs/grid-build-docs -t grid-build-docs:$ISOLATION_ID'
            sh 'docker run --rm -v $(pwd):/project/grid grid-build-docs:$ISOLATION_ID'
        }

        stage("Archive Build artifacts") {
            sh 'mkdir -p build/debs'
            sh 'docker run --rm -v $(pwd)/build/debs/:/debs gridd-installed:${ISOLATION_ID} bash -c "cp /tmp/grid-daemon*.deb /debs && chown -R ${JENKINS_UID} /debs"'
            archiveArtifacts artifacts: '*.tgz, *.zip'
            archiveArtifacts artifacts: 'build/debs/*.deb', allowEmptyArchive: true
            archiveArtifacts artifacts: 'docs/build/html/**, docs/build/latex/*.pdf'
        }

        if (env.BRANCH_NAME == 'master') {
            stage("Build grid-dev image") {
                sh 'docker build -f ci/grid-dev -t hyperledger/grid-dev .'
                sh 'docker push hyperledger/grid-dev'
            }
        }
    }
}
