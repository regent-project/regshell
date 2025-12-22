# Troubleshooting Guide

Common issues and solutions for the Ubuntu test environment deployment.

## Installation Issues

### Minikube won't start

**Problem**: `minikube start` fails

**Solutions**:

1. Check system resources:
```bash
free -h  # Need at least 8GB RAM
nproc    # Need at least 4 CPU cores
df -h    # Need at least 20GB disk space
```

2. Clean previous installation:
```bash
minikube delete
rm -rf ~/.minikube
minikube start --driver=podman  # or docker
```

3. Try different driver:
```bash
# On Fedora
minikube start --driver=podman

# On Ubuntu
minikube start --driver=docker
```

4. Check driver status:
```bash
# For Docker
sudo systemctl status docker
sudo systemctl start docker

# For Podman
podman info
```

### Docker permission denied (Ubuntu)

**Problem**: `permission denied while trying to connect to the Docker daemon socket`

**Solution**:
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Apply group changes without logout
newgrp docker

# Or logout and login again
```

### kubectl connection refused

**Problem**: `The connection to the server localhost:8080 was refused`

**Solution**:
```bash
# Start minikube
minikube start

# Update kubectl context
kubectl config use-context minikube
```

## Deployment Issues

### Pods stuck in Pending state

**Problem**: Pods remain in `Pending` state

**Check**:
```bash
kubectl describe pod <pod-name>
```

**Common causes**:

1. Insufficient resources:
```bash
# Check node resources
kubectl describe node minikube

# Reduce resource requests in manifests/01-ubuntu-nodes-deployment.yaml
# Change:
#   requests:
#     cpu: "100m"    # from 200m
#     memory: "256Mi" # from 512Mi
```

2. Image pull issues:
```bash
# Check if image is available
minikube ssh docker images | grep ubuntu

# Pre-pull image
minikube ssh docker pull ubuntu:22.04
```

### Pods stuck in ContainerCreating

**Problem**: Pods stuck in `ContainerCreating`

**Check**:
```bash
kubectl describe pod <pod-name>
kubectl logs <pod-name> 2>&1
```

**Solutions**:

1. Volume mount issues - Usually auto-resolves, wait 2-3 minutes

2. Network plugin issues:
```bash
kubectl get pods -n kube-system
# Ensure kindnet or CNI pod is running
```

### Pods CrashLoopBackOff

**Problem**: Pods repeatedly crashing

**Diagnose**:
```bash
kubectl logs <pod-name>
kubectl logs <pod-name> --previous  # logs from crashed container
```

**Common causes**:

1. Missing dependencies in startup script - Check pod logs for apt errors

2. SSH daemon failing:
```bash
kubectl exec -it <pod-name> -- /usr/sbin/sshd -T
```

3. Port conflicts - Check if ports 22 or 8080 are being used incorrectly

## Connectivity Issues

### Cannot SSH to nodes

**Problem**: SSH connection refused or timeout

**Diagnose**:
```bash
# Check if pods are running
kubectl get pods

# Check if services exist
kubectl get svc

# Check if ports are accessible
MINIKUBE_IP=$(minikube ip)
nc -zv $MINIKUBE_IP 30101

# Check pod logs
kubectl logs <pod-name>
```

**Solutions**:

1. Wait for pods to be fully ready:
```bash
kubectl wait --for=condition=ready --timeout=300s pod -l app=ubuntu-test-node
```

2. Check if SSH daemon is running inside pod:
```bash
kubectl exec -it <pod-name> -- ps aux | grep sshd
```

3. Test from within minikube:
```bash
minikube ssh
# Inside minikube
curl http://10.244.0.x:8080/health  # use pod IP
```

4. Restart pod if needed:
```bash
kubectl delete pod <pod-name>
# Deployment will recreate it
```

### NodePort not accessible

**Problem**: Cannot access NodePort from host

**Check**:
```bash
# Verify minikube IP
minikube ip

# Check if service has NodePort
kubectl get svc -o wide

# Test from minikube node
minikube ssh
# Inside minikube:
curl localhost:30101  # or appropriate port
```

**Solutions**:

1. On some platforms, use port forwarding:
```bash
kubectl port-forward svc/ubuntu-node-1 2201:22
ssh -p 2201 root@localhost
```

2. Use minikube service:
```bash
minikube service ubuntu-node-1 --url
```

3. Check firewall:
```bash
# Fedora
sudo firewall-cmd --list-all

# Ubuntu
sudo ufw status
```

### Bastion SSH key authentication fails

**Problem**: Cannot connect to bastion with SSH key

**Solutions**:

1. Verify your public key is in the manifest:
```bash
cat ~/.ssh/id_ed25519.pub
# Copy to manifests/03-ssh-bastion.yaml
```

2. Redeploy bastion:
```bash
kubectl delete -f manifests/03-ssh-bastion.yaml
kubectl apply -f manifests/03-ssh-bastion.yaml
```

3. Use password authentication as fallback:
   - Edit `03-ssh-bastion.yaml` to enable password auth

## Health Check Issues

### Readiness probe failing

**Problem**: Pods never become ready

**Check**:
```bash
kubectl describe pod <pod-name>
# Look for readiness probe failures

kubectl logs <pod-name>
```

**Solutions**:

1. Test health endpoint manually:
```bash
kubectl exec -it <pod-name> -- curl http://localhost:8080/ready
```

2. Increase probe timing in manifest:
```yaml
readinessProbe:
  initialDelaySeconds: 60  # increase from 30
  periodSeconds: 15        # increase from 10
```

3. Check if Python server started:
```bash
kubectl exec -it <pod-name> -- ps aux | grep python
kubectl exec -it <pod-name> -- netstat -tulpn | grep 8080
```

## Resource Issues

### Out of memory

**Problem**: Pods being OOMKilled

**Check**:
```bash
kubectl describe pod <pod-name> | grep -i oom
```

**Solutions**:

1. Increase memory limits:
```yaml
resources:
  limits:
    memory: "2Gi"  # increase from 1Gi
```

2. Reduce number of replicas:
```bash
kubectl scale deployment ubuntu-test-nodes --replicas=3
```

3. Increase minikube memory:
```bash
minikube delete
minikube start --memory=12288  # 12GB
```

### CPU throttling

**Problem**: Slow performance

**Check**:
```bash
kubectl top pods
kubectl top nodes
```

**Solutions**:

1. Increase CPU limits in manifest

2. Increase minikube CPUs:
```bash
minikube delete
minikube start --cpus=6
```

## Minikube-Specific Issues

### Minikube IP changes

**Problem**: Minikube IP changed after restart

**Solution**:
```bash
# Update inventory file
MINIKUBE_IP=$(minikube ip)
sed -i "s/192\.168\.[0-9]\+\.[0-9]\+/$MINIKUBE_IP/g" inventory/test-inventory.yaml
```

### Minikube won't stop

**Problem**: `minikube stop` hangs

**Solution**:
```bash
# Force stop
minikube stop --force

# Or delete and recreate
minikube delete
minikube start
```

### Storage issues

**Problem**: Disk space errors

**Check**:
```bash
minikube ssh df -h
```

**Solution**:
```bash
# Clean up unused images
minikube ssh docker system prune -a

# Or increase disk size
minikube delete
minikube start --disk-size=40g
```

## regshell Issues

### Cannot connect to hosts

**Problem**: regshell cannot SSH to hosts

**Diagnose**:
```bash
# Test SSH manually first
ssh -p 30101 root@$(minikube ip)

# Check inventory file
cat inventory/test-inventory.yaml

# Test with increased verbosity
regshell -i inventory/test-inventory.yaml
verbosity 2
cd ubuntu_nodes
ping
```

**Solutions**:

1. Verify inventory file has correct minikube IP

2. Check SSH connectivity manually

3. Ensure pods are ready

### Module execution fails

**Problem**: Modules fail to execute

**Check**:
- Error messages in console output
- Pod logs: `kubectl logs <pod-name>`

**Common issues**:

1. apt module fails - Run with `update_cache=yes` first

2. Python not available - Already installed in base image, check pod logs

## VM-Specific Issues (GNOME Boxes)

### VM out of resources

**Problem**: VM becomes slow or unresponsive

**Solution**:
1. Allocate more resources to VM in GNOME Boxes settings
2. Reduce minikube resource allocation
3. Scale down replicas

### Network not accessible from host

**Problem**: Cannot access minikube from Fedora host

**Check**:
```bash
# Inside VM
minikube ip
ping $(minikube ip)

# From Fedora host
ping <vm-ip>
```

**Solution**: GNOME Boxes uses NAT by default, access from within VM only

## Getting Help

### Collect diagnostic information

```bash
# System info
uname -a
cat /etc/os-release

# Minikube info
minikube version
minikube status
minikube logs

# Kubernetes info
kubectl version
kubectl get nodes
kubectl get pods --all-namespaces
kubectl get events --sort-by='.lastTimestamp'

# Pod details
kubectl describe pods
kubectl logs -l app=ubuntu-test-node --tail=100
kubectl logs ssh-bastion --tail=100
```

### Clean slate reset

If all else fails, start fresh:

```bash
# Complete cleanup
kubectl delete all --all
minikube delete
rm -rf ~/.minikube ~/.kube/config

# On Fedora
./scripts/setup-fedora.sh
./scripts/deploy.sh

# On Ubuntu
./scripts/setup-ubuntu.sh
./scripts/deploy.sh
```

## Common Error Messages

### "error: unable to recognize": Invalid YAML

**Cause**: YAML syntax error in manifest

**Solution**: Validate YAML:
```bash
kubectl apply --dry-run=client -f manifest.yaml
```

### "ImagePullBackOff"

**Cause**: Cannot pull container image

**Solution**:
```bash
# Check image name
kubectl describe pod <pod-name>

# Pre-pull image
minikube ssh docker pull ubuntu:22.04
```

### "CrashLoopBackOff"

**Cause**: Container keeps crashing

**Solution**: Check logs and fix startup script:
```bash
kubectl logs <pod-name> --previous
```

### "Port already in use"

**Cause**: NodePort conflict

**Solution**: Change NodePort in service manifest or free the port
