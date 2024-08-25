#!/bin/bash
kubectl delete --ignore-not-found=true -f kube/elktool-namespace.yml
sleep 15s
kubectl apply -f kube/elktool-namespace.yml

docker build --pull --no-cache . -t elktool-core -f dockerfiles/elktool-core.Dockerfile
docker save elktool-core:latest | k3s ctr images import -
kubectl delete --ignore-not-found=true -f kube/core.yml
sleep 15s
kubectl apply -f kube/core.yml

docker build --pull --no-cache . -t elktool-lifetimes -f dockerfiles/elktool-lifetimes.Dockerfile
docker save elktool-lifetimes:latest | k3s ctr images import -
kubectl delete --ignore-not-found=true -f kube/lifetimes.yml
sleep 15s
kubectl apply -f kube/lifetimes.yml

docker build --pull --no-cache . -t elktool-replicate -f dockerfiles/elktool-replicate.Dockerfile
docker save elktool-replicate:latest | k3s ctr images import -
kubectl delete --ignore-not-found=true -f kube/replicate.yml
sleep 15s
kubectl apply -f kube/replicate.yml

docker build --pull --no-cache . -t elktool-sanitize -f dockerfiles/elktool-sanitize.Dockerfile
docker save elktool-sanitize:latest | k3s ctr images import -
kubectl delete --ignore-not-found=true -f kube/sanitize.yml
sleep 15s
kubectl apply -f kube/sanitize.yml
