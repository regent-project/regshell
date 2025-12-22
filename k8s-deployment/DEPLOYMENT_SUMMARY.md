# Deployment Summary

Complete Kubernetes deployment package for 6-node Ubuntu test environment with SSH bastion host.

## What's Included

This deployment package provides everything needed to recreate your minikube-based Ubuntu test environment on a fresh Fedora or Ubuntu installation.

### Directory Structure

```
k8s-deployment/
├── README.md                    # Comprehensive documentation
├── QUICKSTART.md               # Fast deployment guide
├── DEPLOYMENT_SUMMARY.md       # This file
├── manifests/                  # Kubernetes YAML files
│   ├── 01-ubuntu-nodes-deployment.yaml
│   ├── 02-ubuntu-nodes-services.yaml
│   ├── 03-ssh-bastion.yaml
│   └── 04-individual-node-services.yaml
├── scripts/                    # Automation scripts
│   ├── setup-fedora.sh        # Install prerequisites on Fedora
│   ├── setup-ubuntu.sh        # Install prerequisites on Ubuntu
│   ├── deploy.sh              # Deploy all Kubernetes resources
│   ├── verify.sh              # Verify deployment status
│   ├── test-ssh.sh            # Test SSH connectivity
│   ├── cleanup.sh             # Remove all resources
│   └── ubuntu-vm-test.sh      # All-in-one Ubuntu VM setup
├── inventory/                  # Configuration files
│   └── test-inventory.yaml    # Sample regent-console inventory
└── docs/                       # Documentation
    ├── TESTING.md             # Testing procedures
    └── TROUBLESHOOTING.md     # Common issues and solutions
```

## Components Deployed

### 1. Ubuntu Test Nodes (6 replicas)
- **Image**: ubuntu:22.04
- **Features**:
  - SSH server (port 22)
  - Python3 installed
  - HTTP health check server (port 8080)
  - Network tools (curl, netcat, ping, etc.)
- **Access**: Password authentication (root/testnode123)
- **Resources**: 200m-1000m CPU, 512Mi-1Gi memory per pod
- **Health checks**: Liveness and readiness probes

### 2. SSH Bastion Host
- **Image**: ubuntu:22.04
- **Features**:
  - SSH server with forwarding enabled
  - Public key authentication
  - Can access internal pod network
- **Access**: SSH key authentication (key in manifest)
- **Use case**: Jump host for accessing pods by internal IP

### 3. Services

**ClusterIP Service** (`ubuntu-test-nodes-service`)
- Internal cluster access to all nodes
- Ports: 22 (SSH), 8080 (HTTP)

**NodePort Services** (9 total)
- `ubuntu-test-nodes-nodeport`: Load-balanced access (ports 30022, 30080)
- `ubuntu-node-1` through `ubuntu-node-6`: Individual node access (ports 30101-30106)
- `ssh-bastion`: Bastion access (port 30222)

## Deployment Methods

### Method 1: Fedora Quick Deploy
```bash
cd ~/code/regent-console/k8s-deployment
./scripts/setup-fedora.sh
./scripts/deploy.sh
./scripts/verify.sh
```

### Method 2: Ubuntu Quick Deploy
```bash
cd ~/code/regent-console/k8s-deployment
./scripts/setup-ubuntu.sh
./scripts/deploy.sh
./scripts/verify.sh
```

### Method 3: Ubuntu VM All-in-One
For testing on fresh Ubuntu VM in GNOME Boxes:
```bash
cd ~/code/regent-console/k8s-deployment
./scripts/ubuntu-vm-test.sh
```

### Method 4: Manual Deployment
```bash
# Install minikube and kubectl manually
# Start minikube
minikube start --driver=docker --cpus=4 --memory=8192

# Deploy manifests
kubectl apply -f manifests/01-ubuntu-nodes-deployment.yaml
kubectl apply -f manifests/02-ubuntu-nodes-services.yaml
kubectl apply -f manifests/03-ssh-bastion.yaml
kubectl apply -f manifests/04-individual-node-services.yaml
```

## Access Information

### Get Minikube IP
```bash
MINIKUBE_IP=$(minikube ip)
```

### SSH Access
```bash
# Individual nodes (password: testnode123)
ssh -p 30101 root@$MINIKUBE_IP  # node-1
ssh -p 30102 root@$MINIKUBE_IP  # node-2
ssh -p 30103 root@$MINIKUBE_IP  # node-3
ssh -p 30104 root@$MINIKUBE_IP  # node-4
ssh -p 30105 root@$MINIKUBE_IP  # node-5
ssh -p 30106 root@$MINIKUBE_IP  # node-6

# Bastion (requires SSH key in manifest)
ssh -p 30222 root@$MINIKUBE_IP
```

### HTTP Health Checks
```bash
# Load-balanced health endpoint
curl http://$MINIKUBE_IP:30080/health
curl http://$MINIKUBE_IP:30080/ready
```

## Integration with regent-console

### Update Inventory
```bash
cd ~/code/regent-console
MINIKUBE_IP=$(minikube ip)
sed -i "s/192\.168\.49\.2/$MINIKUBE_IP/g" k8s-deployment/inventory/test-inventory.yaml
```

### Start Console
```bash
./target/release/regent-console -i k8s-deployment/inventory/test-inventory.yaml
```

### Example Usage
```
cd ubuntu_nodes
list
ping
command args="hostname && uname -a"
apt package=vim state=present update_cache=yes
```

## Requirements

### Minimum System Requirements
- **CPU**: 4 cores
- **RAM**: 8GB
- **Disk**: 20GB free space
- **OS**: Fedora or Ubuntu Linux
- **Network**: Internet connection for initial setup

### For Ubuntu VM in GNOME Boxes
- **Host**: Fedora (your current system)
- **VM RAM**: 8GB (8192 MB)
- **VM CPUs**: 4 cores
- **VM Disk**: 20GB minimum
- **VM OS**: Ubuntu 22.04 or later

## Verification

After deployment, verify everything is working:

```bash
./scripts/verify.sh
```

Should show:
- ✓ minikube running
- ✓ All 6/6 replicas ready
- ✓ ssh-bastion running
- ✓ All 9 services present
- ✓ Ports accessible

## Testing

Run comprehensive tests:
```bash
./scripts/test-ssh.sh
```

See `docs/TESTING.md` for detailed testing procedures.

## Troubleshooting

Common issues and solutions are documented in `docs/TROUBLESHOOTING.md`.

Quick diagnostics:
```bash
# Check pod status
kubectl get pods

# Check pod logs
kubectl logs <pod-name>

# Describe pod for events
kubectl describe pod <pod-name>

# Test port connectivity
nc -zv $(minikube ip) 30101
```

## Customization

### Change Node Count
Edit `manifests/01-ubuntu-nodes-deployment.yaml`:
```yaml
spec:
  replicas: 10  # Change from 6 to desired number
```

Then redeploy:
```bash
kubectl apply -f manifests/01-ubuntu-nodes-deployment.yaml
```

### Change SSH Password
Edit `manifests/01-ubuntu-nodes-deployment.yaml`:
```bash
echo 'root:NEW_PASSWORD' | chpasswd
```

Then redeploy.

### Update Bastion SSH Key
Edit `manifests/03-ssh-bastion.yaml` and replace the public key in the `authorized_keys` section with your own key.

### Adjust Resources
Edit resource limits in `manifests/01-ubuntu-nodes-deployment.yaml`:
```yaml
resources:
  requests:
    cpu: "100m"      # Lower for constrained systems
    memory: "256Mi"
  limits:
    cpu: "500m"
    memory: "512Mi"
```

## Maintenance

### View Logs
```bash
# All ubuntu nodes
kubectl logs -l app=ubuntu-test-node --tail=50

# Specific node
kubectl logs ubuntu-test-nodes-<pod-id>

# Bastion
kubectl logs ssh-bastion
```

### Restart Pods
```bash
# Restart specific pod
kubectl delete pod <pod-name>

# Restart all ubuntu nodes
kubectl rollout restart deployment ubuntu-test-nodes
```

### Scale Deployment
```bash
# Scale to 10 nodes
kubectl scale deployment ubuntu-test-nodes --replicas=10

# Scale down to 3 nodes
kubectl scale deployment ubuntu-test-nodes --replicas=3
```

### Update Configuration
```bash
# Apply changes
kubectl apply -f manifests/

# Watch rollout
kubectl rollout status deployment ubuntu-test-nodes
```

## Cleanup

### Remove Deployment Only
```bash
./scripts/cleanup.sh
```

### Remove Minikube Completely
```bash
minikube delete
```

### Clean Everything
```bash
./scripts/cleanup.sh
minikube delete
rm -rf ~/.minikube ~/.kube/config
```

## Backup and Restore

### Export Current Configuration
```bash
# Export all resources
kubectl get deployment ubuntu-test-nodes -o yaml > backup-deployment.yaml
kubectl get svc -o yaml > backup-services.yaml
kubectl get pod ssh-bastion -o yaml > backup-bastion.yaml
```

### Restore from Backup
```bash
kubectl apply -f backup-deployment.yaml
kubectl apply -f backup-services.yaml
kubectl apply -f backup-bastion.yaml
```

## Port Reference

| Service | Port | Purpose |
|---------|------|---------|
| 30101 | SSH | Ubuntu node 1 |
| 30102 | SSH | Ubuntu node 2 |
| 30103 | SSH | Ubuntu node 3 |
| 30104 | SSH | Ubuntu node 4 |
| 30105 | SSH | Ubuntu node 5 |
| 30106 | SSH | Ubuntu node 6 |
| 30022 | SSH | Load-balanced ubuntu nodes |
| 30080 | HTTP | Health check endpoint (load-balanced) |
| 30222 | SSH | Bastion host |

## Quick Reference Commands

```bash
# Get status
kubectl get all
minikube status

# Access minikube node
minikube ssh

# View cluster info
kubectl cluster-info

# Get events
kubectl get events --sort-by='.lastTimestamp'

# Port forward for alternative access
kubectl port-forward svc/ubuntu-node-1 2201:22

# Execute command in pod
kubectl exec -it <pod-name> -- bash

# Copy files to/from pod
kubectl cp file.txt <pod-name>:/tmp/
kubectl cp <pod-name>:/tmp/file.txt ./
```

## Support and Documentation

- **Quick Start**: See `QUICKSTART.md`
- **Full Documentation**: See `README.md`
- **Testing Guide**: See `docs/TESTING.md`
- **Troubleshooting**: See `docs/TROUBLESHOOTING.md`

## Version Information

- **Kubernetes**: Compatible with minikube latest
- **Container Runtime**: Docker or Podman
- **Base Image**: ubuntu:22.04
- **Created**: December 2024

## Success Criteria

Your deployment is successful when:

1. ✅ All 6 ubuntu node pods are Running
2. ✅ Bastion pod is Running
3. ✅ All 9 services are present
4. ✅ SSH connections work to all nodes
5. ✅ HTTP health endpoints respond
6. ✅ regent-console can connect and execute commands
7. ✅ No recurring pod restarts or errors

Run `./scripts/verify.sh` to check all criteria automatically.

---

**Ready to deploy?** Start with `QUICKSTART.md` for the fastest path to a working environment!
