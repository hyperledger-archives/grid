#!/usr/bin/env python3
# Copyright 2018-2022 Cargill Incorporated
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import boto3
import botocore
import os
import requests
import sys
import time
import uuid

action = os.environ["INPUT_ACTION"]
amd_ami = os.environ["INPUT_AMD_AMI_ID"]
amd_instancetype = os.environ["INPUT_AMD_INSTANCE_TYPE"]
arm_ami = os.environ["INPUT_ARM_AMI_ID"]
arm_instancetype = os.environ["INPUT_ARM_INSTANCE_TYPE"]
github_pat = os.environ["INPUT_GH_PERSONAL_ACCESS_TOKEN"]
mode = os.environ["INPUT_MODE"]
region = os.environ["AWS_REGION"]
repo = os.environ["GITHUB_REPOSITORY"]
securitygroup = os.environ["INPUT_SECURITY_GROUP_ID"]
subnet = os.environ["INPUT_SUBNET"]

ec2client = boto3.client('ec2', region_name=region)

def create_instance(ami, instancetype, label, userdata):
    response = ec2client.run_instances(
        BlockDeviceMappings=[
            {
                'DeviceName': '/dev/xvda',
                'Ebs': {
                    'DeleteOnTermination': True,
                    'VolumeSize': 30,
                    'VolumeType': 'gp2'
                },
            },
        ],
        ImageId=ami,
        InstanceType=instancetype,
        MaxCount=1,
        MinCount=1,
        Monitoring={
            'Enabled': False
        },
        SecurityGroupIds=(securitygroup,),
        SubnetId=subnet,
        TagSpecifications=[
            {
                "ResourceType": "instance",
                'Tags': [
                    {'Key': 'Name', 'Value': label },
                ]
            },
        ],
        UserData=userdata,
    )
    return response

def get_instances_from_tag(tag):
    instance_ids = []
    tag_filter = [
        {'Name':'tag:Name','Values': [tag]},
    ]

    response = ec2client.describe_instances(Filters=tag_filter)

    for reservation in response["Reservations"]:
        for instance in reservation["Instances"]:
            instance_ids.append(instance["InstanceId"])
    return instance_ids

def get_regtoken():
    try:
        headers = {'Authorization': "token {}".format(github_pat.strip())}
        r = requests.post(f"https://api.github.com/repos/{repo}/actions/runners/registration-token", headers=headers)
        return r.json()["token"]
    except:
        print("ERROR: Unable to get GHA self-hosted registration token")
        sys.exit(1)

def make_label():
    return str(uuid.uuid1()).split("-")[0]

def terminate_instances(tag):
    instances_to_terminate = get_instances_from_tag(tag)
    print(instances_to_terminate)
    try:
        response = ec2client.terminate_instances(
            InstanceIds=(instances_to_terminate), DryRun=True
        )
    except botocore.exceptions.ClientError as e:
        if 'DryRunOperation' not in str(e):
            raise
    try:
        response = ec2client.terminate_instances(
            InstanceIds=(instances_to_terminate), DryRun=False
        )
        print("Termination was successful")
    except botocore.exceptions.ClientError as e:
        print(e)

if action == "start":
    reg_token = get_regtoken()
    label = make_label()
    print(f"Creating instances with tag {label}")
    if mode == "buildx":
        arm_userdata=""
        arminstance=create_instance(arm_ami, arm_instancetype, label, arm_userdata)
        arm_private_ip = arminstance['Instances'][0]['PrivateIpAddress']
        print("Started ARM instance %s at %s" % (arminstance['Instances'][0]['InstanceId'], arm_private_ip))
        print("Sleeping for 20s so %s will be ready" % arminstance['Instances'][0]['InstanceId'])
        time.sleep(20)

        buildx_userdata=f"""
        echo "{arm_private_ip} buildx" >> /etc/hosts
        DOCKER_HOST=tcp://buildx:2375 docker buildx create --name cluster
        docker buildx create --name cluster --append
        docker buildx use cluster
        docker buildx inspect --bootstrap
        """
    else:
        buildx_userdata=""

    amd_userdata=f"""#!/bin/bash
        {buildx_userdata}
        mkdir /tmp/actions-runner && cd /tmp/actions-runner
        curl -o actions-runner-linux-x64-2.288.1.tar.gz -L https://github.com/actions/runner/releases/download/v2.288.1/actions-runner-linux-x64-2.288.1.tar.gz
        tar xzf ./actions-runner-linux-x64-2.288.1.tar.gz
        RUNNER_ALLOW_RUNASROOT=1 ./config.sh --url https://github.com/{repo} --token {reg_token} --labels {label} --ephemeral --unattended
        RUNNER_ALLOW_RUNASROOT=1 ./run.sh
    """

    print(f"::set-output name=label::{label}")
    amdinstance=create_instance(amd_ami, amd_instancetype, label, amd_userdata)

if action == "stop":
    terminate_instances(os.environ["INPUT_LABEL"])
