# Kubernetes Ubuntu Test Environment Deployment

This directory contains all configuration files and scripts to deploy a 6-node Ubuntu test environment with SSH bastion host on minikube.

## Prerequisites

### For Fedora
```bash
sudo dnf install -y curl wget git
```

### For Ubuntu
```bash
sudo apt update
sudo apt install -y curl wget git
```

## Quick Start

1. Run the setup script for your OS:
   ```bash
   # For Fedora
   ./scripts/setup-fedora.sh
   
   # For Ubuntu
   ./scripts/setup-ubuntu.sh
   ```

2. Deploy the Kubernetes resources:
   ```bash
   ./scripts/deploy.sh
   ```

3. Verify deployment:
   ```bash
   ./scripts/verify.sh
   ```

## Architecture

### Components

**6 Ubuntu Test Nodes** (`ubuntu-test-nodes` deployment)
- Ubuntu 22.04 containers with SSH access
- Python3 for automation tasks
- HTTP health check server on port 8080
- Credentials: `root/testnode123`
- NodePort access: 30101-30106 (individual nodes)
- NodePort access: 30022 (load-balanced)

**SSH Bastion Host** (`ssh-bastion` pod)
- Jump host for accessing test nodes
- SSH forwarding enabled
- Public key authentication
- NodePort access: 30222

### Network Layout

```
                    ┌─────────────────┐
                    │   Minikube IP   │
                    │  (192.168.49.2) │
                    └────────┬────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
    Port 30222          Port 30022         Ports 30101-30106
          │                  │                  │
    ┌─────▼──────┐    ┌─────▼──────┐    ┌─────▼──────┐
    │  Bastion   │    │   Pool     │    │ Individual │
    │            │    │ Access     │    │   Nodes    │
    └────────────┘    └─────┬──────┘    └────────────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
         Pod Network                 Pod Network
        (10.244.0.x)               (10.244.0.x)
              │                           │
      ┌───────┴────────┐         ┌───────┴────────┐
      │  ubuntu-node-1 │   ...   │  ubuntu-node-6 │
      │  (SSH on 22)   │         │  (SSH on 22)   │
      │  (HTTP on 8080)│         │  (HTTP on 8080)│
      └────────────────┘         └────────────────┘
```

### Access Methods

**Direct NodePort Access (from host)**
```bash
# Access bastion
ssh -p 30222 root@$(minikube ip)

# Access individual nodes directly
ssh -p 30101 root@$(minikube ip)  # node-1
ssh -p 30102 root@$(minikube ip)  # node-2
...
ssh -p 30106 root@$(minikube ip)  # node-6
```

**Via Bastion (jump host)**
```bash
# From bastion, access nodes by pod IP
ssh root@<pod-ip>
```

**Using kubectl port-forward**
```bash
# Forward bastion to localhost
kubectl port-forward pod/ssh-bastion 2222:22

# Then connect
ssh -p 2222 root@localhost
```

## Directory Structure

```
k8s-deployment/
├── README.md                    # This file
├── manifests/
│   ├── 01-ubuntu-nodes-deployment.yaml   # 6-node deployment
│   ├── 02-ubuntu-nodes-services.yaml     # ClusterIP and NodePort services
│   ├── 03-ssh-bastion.yaml              # Bastion pod and service
│   └── 04-individual-node-services.yaml  # Per-node NodePort services
├── scripts/
│   ├── setup-fedora.sh          # Install minikube on Fedora
│   ├── setup-ubuntu.sh          # Install minikube on Ubuntu
│   ├── deploy.sh                # Deploy all manifests
│   ├── verify.sh                # Verify deployment
│   ├── cleanup.sh               # Remove all resources
│   └── test-ssh.sh              # Test SSH connectivity
├── inventory/
│   └── test-inventory.yaml      # Sample inventory for regent-console
└── docs/
    ├── TESTING.md               # Testing guide
    └── TROUBLESHOOTING.md       # Common issues and solutions
```

## Configuration Details

### Resource Limits
Each node is configured with:
- CPU Request: 200m
- CPU Limit: 1000m (1 core)
- Memory Request: 512Mi
- Memory Limit: 1Gi

### Health Checks
- **Liveness Probe**: HTTP GET /health on port 8080 (60s delay, 30s interval)
- **Readiness Probe**: HTTP GET /ready on port 8080 (30s delay, 10s interval)

### Credentials
- **Ubuntu Nodes**: `root/testnode123` (password)
- **Bastion**: Public key authentication (key in manifest)

## Usage with regent-console

```bash
# Start regent-console with inventory
regent-console -i inventory/test-inventory.yaml

# Or connect to specific nodes
regent-console ubuntu-nodes

# Example commands
cd ubuntu-nodes
list
command module args="uname -a"
apt package=nginx state=present
```

## Maintenance

### Update Node Password
Edit `manifests/01-ubuntu-nodes-deployment.yaml` and change:
```bash
echo 'root:NEW_PASSWORD' | chpasswd
```

### Scale Nodes
```bash
kubectl scale deployment ubuntu-test-nodes --replicas=10
```

### Update SSH Keys for Bastion
Edit `manifests/03-ssh-bastion.yaml` and update the authorized_keys section.

## Cleanup

```bash
# Remove all resources
./scripts/cleanup.sh

# Remove minikube cluster entirely
minikube delete
```

## Testing on Ubuntu VM

To test on an Ubuntu VM in GNOME Boxes:

1. Create Ubuntu VM with at least:
   - 4 CPU cores
   - 8GB RAM
   - 20GB disk

2. Install and setup:
   ```bash
   git clone <repo>
   cd regent-console/k8s-deployment
   ./scripts/setup-ubuntu.sh
   ./scripts/deploy.sh
   ```

3. Verify:
   ```bash
   ./scripts/verify.sh
   ./scripts/test-ssh.sh
   ```

## Troubleshooting

See `docs/TROUBLESHOOTING.md` for common issues and solutions.

## License

Same as regent-console project.
