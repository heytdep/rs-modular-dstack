#!/bin/bash

if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <CLUSTER> <PUBKEY> <SECRET>"
  exit 1
fi

CLUSTER="$1"
PUBKEY="$2"
SECRET="$3"

# TODO: Is there a way for env vars to be inferred at runtime? probably yes.
generate_newyork_yml() {
  echo "Generating they pod yml definition ..."
  cat <<EOF > newyork.yml
apiVersion: v1
kind: Pod
metadata:
  name: new-york-pod
  labels:
    app: new-york
spec:
  containers:
    - name: new-york-container
      image: new-york-image:latest
      env:
        - name: CLUSTER
          value: "$CLUSTER"
    - name: ping-host-container
      image: ping-host-image:latest
  hostNetwork: true
EOF
  echo "newyork.yml created successfully."
}


# Services that need to run guest-side are guest microservice (both host-facing and guest-facing) and the actual user application.
build_images() {
  if ! podman image exists localhost/ping-host-image:latest; then
    echo "Building app image ..."
    podman build -f examples/ping-host/Dockerfile -t localhost/ping-host-image:latest .
  else
    echo "app's image already exists."
  fi

  if ! podman image exists localhost/new-york-image:latest; then
    echo "Building new-york image ..."
    podman build -f new-york/Dockerfile -t localhost/new-york-image:latest .
  else
    echo "new-york-image already exists."
  fi
}

# Build new-york host on host machine
build_and_run_rust() {
  echo "Building Rust application..."
  pushd new-york
  cargo build --release
  popd

  echo "Running Rust application..."
  CLUSTER="$CLUSTER" SECRET="$SECRET" ./new-york/target/release/host &
}

# Upload to flashbox!
send_curl_requests() {
  echo "Sending pod to flashbox..."

  # Do we need this at all? Doing just to be sure but the pod yml should be inferring the vars.
  echo "CLUSTER=$CLUSTER" > env
  echo "PUBKEY=$PUBKEY" >> env

  # Upload
  curl -X POST -F "pod.yaml=@newyork.yml" -F "env=@env" http://localhost:24070/upload
  echo "Pod uploaded."

  # Start
  curl -X POST http://localhost:24070/start
  echo "Pod started successfully."
}

generate_newyork_yml
build_images
build_and_run_rust
send_curl_requests

echo "All tasks completed successfully!"
