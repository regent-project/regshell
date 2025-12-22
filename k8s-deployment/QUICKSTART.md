# Quick Start Guide

Get your 6-node Ubuntu test environment running in under 10 minutes!

## Prerequisites

- Fedora or Ubuntu Linux
- 8GB RAM minimum
- 4 CPU cores minimum
- 20GB free disk space
- Internet connection

## Installation Steps

### Step 1: Navigate to deployment directory

```bash
cd ~/code/regshell/k8s-deployment
```

### Step 2: Run setup script

**On Fedora:**
```bash
./scripts/setup-fedora.sh
```

**On Ubuntu:**
```bash
./scripts/setup-ubuntu.sh
```

⏱️ This takes ~5-10 minutes depending on your internet speed.

### Step 3: Deploy the environment

```bash
./scripts/deploy.sh
```

⏱️ This takes ~3-5 minutes for pods to be ready.

### Step 4: Verify deployment

```bash
./scripts/verify.sh
```

You should see all checks passing! ✓

## Quick Access

Get your minikube IP:
```bash
MINIKUBE_IP=$(minikube ip)
echo $MINIKUBE_IP
```

### SSH to nodes

Password for all nodes: `testnode123`

```bash
# Node 1
ssh -p 30101 root@$MINIKUBE_IP

# Node 2
ssh -p 30102 root@$MINIKUBE_IP

# ... and so on for nodes 3-6
```

### Test SSH connectivity

```bash
./scripts/test-ssh.sh
```

## Using with regshell

### Update inventory file

```bash
cd ~/code/regshell
MINIKUBE_IP=$(minikube ip)
sed -i "s/192\.168\.49\.2/$MINIKUBE_IP/g" k8s-deployment/inventory/test-inventory.yaml
```

### Start regshell

```bash
# Build if not already built
cargo build --release

# Start with inventory
./target/release/regshell -i k8s-deployment/inventory/test-inventory.yaml
```

### Try it out

In the regshell prompt:

```
cd ubuntu_nodes
list
ping
command args="hostname && uname -a"
```

## Testing on Ubuntu VM (GNOME Boxes)

### Create VM

1. Open GNOME Boxes
2. Create new Ubuntu VM:
   - **Name**: ubuntu-k8s-test
   - **RAM**: 8192 MB (8GB)
   - **CPUs**: 4 cores
   - **Disk**: 20GB minimum

3. Install Ubuntu (minimal or desktop)

### Clone and deploy

Inside the VM:

```bash
# Install git if needed
sudo apt install git

# Clone your repo (adjust URL as needed)
git clone <your-repo-url>
cd regshell/k8s-deployment

# Run setup
./scripts/setup-ubuntu.sh

# Deploy
./scripts/deploy.sh

# Verify
./scripts/verify.sh
```

## Troubleshooting

### Pods stuck in Pending?

Wait 2-3 minutes. Still stuck?

```bash
kubectl describe pod <pod-name>
```

Look for resource issues. Might need to increase VM resources.

### Can't SSH to nodes?

```bash
# Check if pods are ready
kubectl get pods

# Check if services are running
kubectl get svc

# Test port accessibility
nc -zv $(minikube ip) 30101
```

### Docker permission denied (Ubuntu)?

```bash
newgrp docker
# Or logout and login again
```

### More help

See detailed guides:
- `docs/TESTING.md` - Testing procedures
- `docs/TROUBLESHOOTING.md` - Common issues

## What's Next?

- **Scale up**: `kubectl scale deployment ubuntu-test-nodes --replicas=10`
- **Add nodes**: Edit `manifests/01-ubuntu-nodes-deployment.yaml`
- **Change passwords**: Edit deployment manifest and redeploy
- **Use bastion**: Add your SSH key to `manifests/03-ssh-bastion.yaml`

## Cleanup

Remove all resources:
```bash
./scripts/cleanup.sh
```

Remove minikube completely:
```bash
minikube delete
```

## Architecture Overview

```
Your Machine / VM
    │
    ├─ minikube (Kubernetes)
    │   │
    │   ├─ 6 Ubuntu Pods (ubuntu-test-nodes)
    │   │   ├─ SSH on port 22
    │   │   └─ HTTP health on port 8080
    │   │
    │   └─ 1 Bastion Pod (ssh-bastion)
    │       └─ SSH on port 22
    │
    └─ NodePort Access
        ├─ 30101-30106 → Individual nodes
        ├─ 30022 → Load-balanced nodes
        └─ 30222 → Bastion
```

## Support

For issues or questions:
1. Check `docs/TROUBLESHOOTING.md`
2. Run `./scripts/verify.sh` for diagnostics
3. Check logs: `kubectl logs <pod-name>`

Happy testing! 🚀
