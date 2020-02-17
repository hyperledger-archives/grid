# Running the Gameroom Demo in Kubernetes

This procedure explains how to run the
[Gameroom demo](https://github.com/Cargill/splinter/tree/master/examples/gameroom)
with
[Kubernetes](https://kubernetes.io/docs/concepts/overview/what-is-kubernetes/).
This environment uses [Minikube](https://kubernetes.io/docs/setup/minikube/) to
deploy two containerized Splinter nodes in a local Kubernetes cluster inside a
virtual machine (VM) on your computer. This single pod deployment for each
organization isn't intended to depict a production-like scenario. This is to
demonstate the Kubernetes primitives.

This procedure walks you through:

* Generating keys and updating the key registry
* Creating ConfigMaps for the key registry and node registry
* Starting Gameroom
* Creating users and logging in to the web app
* Playing tic-tac-toe

## Prerequisites

You'll need Minikube installed and started and kubectl to complete this
walkthrough. For installation instructions, see the
[Minikube installation documentation](https://kubernetes.io/docs/tasks/tools/install-minikube/).

## Deploy Gameroom

### Step 1: Generate keys

1. Run a Job to generate user keys.

   `$ kubectl apply -f https://raw.githubusercontent.com/Cargill/splinter/master/docker/kubernetes/create-gameroom-keys.yaml`

1. View the output from the Job.

  ```
  $ jobpod=$(kubectl get pods --selector=job-name=gameroom-keys --output=jsonpath='{.items[*].metadata.name}')
  $ kubectl logs $jobpod
  ```

   The output will be similar to this example:

  ```
  alice.priv: 36a7e8cc7c81fdc7c7278aa157a732c283b9ba0e2ee4ec363992d3d96bad8207
  alice.pub: 031bc8f3a326766ce628295ad6f01c49fd416306a0cf55cca42a6b87e3eccc3c8f
  bob.priv: 8013752e26576e27e31105f6cf0f4537dc8a381a5b347a09959292cb4b17b388
  bob.pub: 029d011c4c4f058b2b5a4cdff1dbe1749809cb121eb45dc766292a26d3a9b84fcd
  ```

1. Copy these values to a scratchpad or keep them available in your terminal,
   because you'll be using them a few times in the walkthrough.

### Step 2: Update the key registry template

1. Download the key registry template, [key-registry.yaml](https://raw.githubusercontent.com/Cargill/splinter/master/docker/kubernetes/key-registry.yaml).

1. Edit `key-registry.yaml` to add the public key values created above.

   `$ vim key-registry.yaml`

1. Replace the lines that say `alice.pub` with the corresponding key data generated
   from the job run in the previous step.

1. Next, do the same with the `bob.pub` lines.

1. Make sure that the YAML is correctly formatted. It should look similar to this example:

   ```
   ---
   031bc8f3a326766ce628295ad6f01c49fd416306a0cf55cca42a6b87e3eccc3c8f:
     public_key: 031bc8f3a326766ce628295ad6f01c49fd416306a0cf55cca42a6b87e3eccc3c8f
     associated_node_id: acme
     metadata:
       gameroom/first-name: alice
       gameroom/organization: Acme
   029d011c4c4f058b2b5a4cdff1dbe1749809cb121eb45dc766292a26d3a9b84fcd:
     public_key: 029d011c4c4f058b2b5a4cdff1dbe1749809cb121eb45dc766292a26d3a9b84fcd
     associated_node_id: bubba
     metadata:
       gameroom/first-name: bob
       gameroom/organization: Bubba Bakery
   ```

### Step 3: Create ConfigMaps for the key registry and node registry

1. Generate a ConfigMap for the key registry.

    `$ kubectl create configmap key-registry --from-file key-registry.yaml`

1. Download the node registry file: [node-registry.yaml](https://raw.githubusercontent.com/Cargill/splinter/master/docker/kubernetes/node-registry.yaml).

1. Generate a ConfigMap for the node registry.

    `$ kubectl create configmap node-registry --from-file node-registry.yaml`

1. Verify that the ConfigMaps were created.

    ```
    $ kubectl get cm
    NAME            DATA   AGE
    key-registry    1      30s
    node-registry   1      2m43s
    ```

1. You can inspect the values of the ConfigMaps by running
    `kubectl describe cm <configmapname>`. For example:

    ```
    $ kubectl describe cm node-registry
    Name:         node-registry
    Namespace:    default
    Labels:       <none>
    Annotations:  <none>

    Data
    ====
    node-registry.yaml:
    ----
    ---
    - identity: "acme"
      endpoint: "tls://acme.default.svc.cluster.local:8044"
      display_name: "Acme"
      metadata:
        organization: "Acme Corporation"

    - identity: "bubba"
      endpoint: "tls://bubba.default.svc.cluster.local:8044"
      display_name: "Bubba Bakery"
      metadata:
        organization: "Bubba Bakery"

    Events:  <none>
    ```

### Step 4: Start Gameroom

1. Apply the `arcade.yaml` manifest.

   ```
   $ kubectl apply -f https://raw.githubusercontent.com/Cargill/splinter/master/docker/kubernetes/arcade.yaml
   
   deployment.apps/acme created
   service/acme-splinterd created
   service/acme-http created
   deployment.apps/bubba created
   service/bubba-splinterd created
   service/bubba-http created
   ```

1. Verify that the Pods started correctly:

   ```
   $ kubectl get pods
   NAME                     READY   STATUS      RESTARTS   AGE
   acme-7575f75d6d-trddh    5/5     Running     0          6m10s
   bubba-5d5f554fdb-l9nr4   5/5     Running     0          6m9s
   gameroom-keys-l42fz      0/1     Completed   0          56m
   ```

### Step 5: Create users and log in to the web apps

1. Run `minikube service` to open the Acme web app.

   ```
   $ minikube service acme-http
   |-----------|-----------|-------------|---------------------------|
   | NAMESPACE |   NAME    | TARGET PORT |            URL            |
   |-----------|-----------|-------------|---------------------------|
   | default   | acme-http | http        | http://192.168.64.3:30160 |
   |-----------|-----------|-------------|---------------------------|
   Opening service default/acme-http in default browser...
   ```
   ![alt text](images/acme-1-launch.png "Acme gameroom homepage")

1. In the upper left, click **Register**.

1. Fill out the registration form with values you'll remember. For the
   `Private Key` field, enter the private key information generated for Alice in
   Step 1.

   ![alt text](images/acme-2-register.png "Acme registration page")

   You're logged in as Alice now.

   ![alt text](images/acme-3-loggedin.png "Alice logged in page")

1. Repeat these steps to launch the Bubba Bakery web app and register with Bob's
   private key.

   `$ minikube service bubba-http`

   ![alt text](images/bubba-1-launch.png "Bubba gameroom homepage")

   ![alt text](images/bubba-2-register.png "Bubba registration page")

   ![alt text](images/bubba-3-loggedin.png "Bob logged in page")

### Step 6: Create a gameroom

1. Switch back to the Acme web app. Click the `+` next to `My Gamerooms`.
   Select Bubba Bakery from the dropdown menu and give your new gameroom a name.
   Then click **Send**.

   ![alt text](images/acme-4-newgameroom.png "Creating a new gameroom")

1. After clicking Send, you'll see a green notification indicating that the
   invitation has been successfully sent.

   ![alt text](images/acme-5-invitationsent.png "Invitation successfully sent")

1. Click **Invitations** in the bottom left, then **SENT**. The invitation is
   visible in the list.

   ![alt text](images/acme-6-invitationlist.png "Alice's sent invitation list")

1. Switch to the Bubba Bakery web app. You'll see that the bell in the upper right
   has an activity indicator. Click this to view the notification, then click the
   notification to view the invitation.

   ![alt text](images/bubba-4-invitationnotification.png "Bob's invitation notification")

1. Click **Accept** to accept the invitation. After a brief notification, the
   invitation will disappear.

   ![alt text](images/bubba-5-invitation.png "Bob's invitation")

1. Click the newly created gameroom in the left sidebar to view the gameroom.

   ![alt text](images/bubba-6-gameroom.png "Bob's view of the new gameroom")

### Step 7: Play tic-tac-toe

1. Switch back to the Acme web app. Click the new gameroom in the left sidebar.
   Then click the **New Game** button to start a new game of tic-tac-toe. Enter a
   name for your game and click **Send**.

   ![alt text](images/acme-7-creategame.png "Alice creates a new game")

1. Click anywhere on the game to join, then select a square to begin.

   ![alt text](images/acme-8-joingame.png "Alice joins the new game")

   ![alt text](images/acme-9-takespace.png "Alice takes a space")

1. Switch to the Bubba Bakery web app to continue playing.

   ![alt text](images/bubba-7-joingame.png "Bob joins the new game")

   ![alt text](images/bubba-8-takespace.png "Bob takes a space")

You can continue playing this game, start new games, or create new gamerooms.

## Step 8: Stop the Gameroom Demo

Once you're finished playing, clean up by deleting the local Minikube cluster.
This will remove all data and state. If you want to run Gameroom again, you'll
have to start at the beginning of the walkthrough.

`$ minikube delete`
