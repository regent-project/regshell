# Testing Guide

This guide provides instructions for testing the Ubuntu test environment deployment.

## Prerequisites

1. Minikube is running
2. All pods are deployed and ready
3. Services are accessible

Verify with:
```bash
./scripts/verify.sh
```

## Basic SSH Testing

### Test Direct SSH Access

```bash
# Get minikube IP
MINIKUBE_IP=$(minikube ip)

# Test individual nodes (password: testnode123)
ssh -p 30101 root@$MINIKUBE_IP "hostname && uname -a"
ssh -p 30102 root@$MINIKUBE_IP "hostname && uname -a"
ssh -p 30103 root@$MINIKUBE_IP "hostname && uname -a"
ssh -p 30104 root@$MINIKUBE_IP "hostname && uname -a"
ssh -p 30105 root@$MINIKUBE_IP "hostname && uname -a"
ssh -p 30106 root@$MINIKUBE_IP "hostname && uname -a"
```

### Test Bastion Access

```bash
# Ensure your SSH key is in manifests/03-ssh-bastion.yaml
ssh -p 30222 root@$MINIKUBE_IP

# Once inside bastion, you can access nodes by pod IP
ssh root@<pod-ip>
```

## Testing with regshell

### Setup

1. Build regshell if not already built:
```bash
cd /path/to/regshell
cargo build --release
```

2. Update the inventory file with your minikube IP:
```bash
MINIKUBE_IP=$(minikube ip)
sed -i "s/192.168.49.2/$MINIKUBE_IP/g" k8s-deployment/inventory/test-inventory.yaml
```

### Basic Console Tests

```bash
# Start regshell with inventory
./target/release/regshell -i k8s-deployment/inventory/test-inventory.yaml

# In the console:
cd ubuntu_nodes
list
```

### Module Testing

#### Test ping module
```
cd ubuntu_nodes
ping
```

Expected output: All hosts should respond with "pong"

#### Test command module
```
cd ubuntu_nodes
command args="uname -a"
```

Expected output: Linux kernel information for each node

#### Test apt module
```
cd ubuntu-node-1
apt package=vim state=present update_cache=yes
```

Expected output: vim should be installed

#### Test debug module
```
cd ubuntu_nodes
debug msg="Hello from {{ ansible_hostname }}"
```

Expected output: Debug message with each hostname

### Variable Testing

```
set package_name nginx
cd ubuntu-node-1
apt package={{ package_name }} state=present
```

### Parallel Execution Testing

```
# Set high parallelism
forks 10

# Run command on all nodes
cd ubuntu_nodes
command args="sleep 2 && hostname"
```

All 6 nodes should execute in parallel.

## Network Testing

### Test HTTP Health Endpoints

```bash
MINIKUBE_IP=$(minikube ip)

# Test health endpoint (load-balanced across all nodes)
curl http://$MINIKUBE_IP:30080/health

# Test ready endpoint
curl http://$MINIKUBE_IP:30080/ready
```

### Test Pod-to-Pod Communication

From one node, test connectivity to another:

```bash
# SSH to node 1
ssh -p 30101 root@$(minikube ip)

# Inside the pod, test connectivity
# Get other pod IPs first:
# kubectl get pods -o wide

# Test SSH to another pod
ssh root@<other-pod-ip>

# Test HTTP to another pod
curl http://<other-pod-ip>:8080/health
```

## Performance Testing

### Connection Speed Test

```bash
time ssh -p 30101 root@$(minikube ip) "exit"
```

Should complete in < 2 seconds.

### Parallel Task Execution

```bash
# In regshell
forks 6
cd ubuntu_nodes
command args="dd if=/dev/zero of=/tmp/test bs=1M count=100"
```

Monitor with:
```bash
kubectl top pods
```

## Troubleshooting Tests

### Check Pod Logs

```bash
# Ubuntu nodes
kubectl logs -l app=ubuntu-test-node --tail=50

# Bastion
kubectl logs ssh-bastion --tail=50
```

### Check Service Endpoints

```bash
kubectl get endpoints
```

### Test Network Policies

```bash
# From inside a pod
kubectl exec -it ubuntu-test-nodes-<pod-id> -- bash

# Test DNS
nslookup ubuntu-test-nodes-service

# Test service connectivity
curl http://ubuntu-test-nodes-service:8080/health
```

### Port Forwarding Alternative

If NodePort access isn't working:

```bash
# Forward individual node
kubectl port-forward ubuntu-test-nodes-<pod-id> 2201:22

# Then connect
ssh -p 2201 root@localhost
```

## Load Testing

### Stress Test SSH Connections

```bash
for i in {1..10}; do
  ssh -p 30101 root@$(minikube ip) "hostname" &
done
wait
```

### Stress Test HTTP Endpoints

```bash
# Install apache bench if not available
# sudo dnf install httpd-tools  # Fedora
# sudo apt install apache2-utils  # Ubuntu

ab -n 1000 -c 10 http://$(minikube ip):30080/health
```

## Automated Test Script

Create a test script:

```bash
#!/bin/bash
set -e

MINIKUBE_IP=$(minikube ip)

echo "Testing all nodes..."
for i in {1..6}; do
    PORT=$((30100 + i))
    echo -n "Node $i: "
    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 \
        -p $PORT root@$MINIKUBE_IP \
        "hostname" 2>/dev/null || echo "FAILED"
done

echo "Testing health endpoints..."
curl -s http://$MINIKUBE_IP:30080/health | grep -q "healthy" && echo "Health check: PASSED" || echo "Health check: FAILED"

echo "All tests complete!"
```

## VM-Specific Testing (GNOME Boxes Ubuntu)

When testing on Ubuntu VM in GNOME Boxes:

1. Ensure VM has enough resources:
```bash
# Check available resources
free -h
nproc
df -h
```

2. Monitor resource usage during tests:
```bash
watch -n 1 'kubectl top nodes && kubectl top pods'
```

3. Test from host to VM if network is bridged:
```bash
# From your Fedora host, if VM is accessible
ssh user@vm-ip
# Then test from inside VM
ssh -p 30101 root@$(minikube ip)
```

## Success Criteria

A successful deployment should:

1. ✓ All 6 Ubuntu pods running
2. ✓ Bastion pod running
3. ✓ All NodePort services accessible
4. ✓ SSH connections working
5. ✓ HTTP health endpoints responding
6. ✓ regshell can connect and execute commands
7. ✓ Parallel execution works correctly
8. ✓ No pod restarts or errors in logs

## Cleanup After Testing

```bash
./scripts/cleanup.sh
```

Or to completely remove minikube:
```bash
minikube delete
```
