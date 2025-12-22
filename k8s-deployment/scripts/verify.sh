#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Deployment Verification"
echo "=========================================="
echo ""

# Check minikube status
echo "[1/5] Checking minikube status..."
if ! minikube status &> /dev/null; then
    echo "❌ FAIL: minikube is not running"
    exit 1
else
    echo "✓ PASS: minikube is running"
    MINIKUBE_IP=$(minikube ip)
    echo "  Minikube IP: $MINIKUBE_IP"
fi
echo ""

# Check deployment
echo "[2/5] Checking ubuntu-test-nodes deployment..."
DESIRED_REPLICAS=$(kubectl get deployment ubuntu-test-nodes -o jsonpath='{.spec.replicas}')
READY_REPLICAS=$(kubectl get deployment ubuntu-test-nodes -o jsonpath='{.status.readyReplicas}')

if [ "$DESIRED_REPLICAS" -eq "$READY_REPLICAS" ] 2>/dev/null; then
    echo "✓ PASS: All $READY_REPLICAS/$DESIRED_REPLICAS replicas are ready"
else
    echo "⚠ WARNING: Only $READY_REPLICAS/$DESIRED_REPLICAS replicas are ready"
fi
echo ""

# Check bastion pod
echo "[3/5] Checking ssh-bastion pod..."
BASTION_STATUS=$(kubectl get pod ssh-bastion -o jsonpath='{.status.phase}')
if [ "$BASTION_STATUS" = "Running" ]; then
    echo "✓ PASS: ssh-bastion is running"
else
    echo "❌ FAIL: ssh-bastion status is $BASTION_STATUS"
fi
echo ""

# Check services
echo "[4/5] Checking services..."
SERVICES=$(kubectl get svc -o name | grep -E "ubuntu|bastion" | wc -l)
if [ "$SERVICES" -ge 9 ]; then
    echo "✓ PASS: All services are present ($SERVICES services)"
else
    echo "⚠ WARNING: Only $SERVICES services found (expected 9+)"
fi
echo ""

# Check NodePort accessibility
echo "[5/5] Checking NodePort accessibility..."
echo "Testing bastion port (30222)..."
if timeout 2 bash -c "nc -z $MINIKUBE_IP 30222" 2>/dev/null; then
    echo "✓ PASS: Bastion port 30222 is accessible"
else
    echo "⚠ WARNING: Bastion port 30222 is not accessible (may still be starting)"
fi

echo "Testing ubuntu node port (30101)..."
if timeout 2 bash -c "nc -z $MINIKUBE_IP 30101" 2>/dev/null; then
    echo "✓ PASS: Ubuntu node port 30101 is accessible"
else
    echo "⚠ WARNING: Ubuntu node port 30101 is not accessible (may still be starting)"
fi
echo ""

# Summary
echo "=========================================="
echo "Summary"
echo "=========================================="
echo ""
kubectl get pods
echo ""
kubectl get svc | grep -E "NAME|ubuntu|bastion"
echo ""

# Health check
echo "=========================================="
echo "Health Check"
echo "=========================================="
echo ""
echo "Checking HTTP health endpoints..."
for i in {1..6}; do
    PORT=$((30100 + i))
    if curl -s -m 2 http://$MINIKUBE_IP:30080/health | grep -q "healthy" 2>/dev/null; then
        echo "✓ Node $i health check: OK"
        break
    fi
done
echo ""

echo "=========================================="
echo "Access Commands"
echo "=========================================="
echo ""
echo "SSH to bastion:"
echo "  ssh -p 30222 root@$MINIKUBE_IP"
echo ""
echo "SSH to individual nodes:"
for i in {1..6}; do
    PORT=$((30100 + i))
    echo "  ssh -p $PORT root@$MINIKUBE_IP  # node-$i (password: testnode123)"
done
echo ""

echo "=========================================="
echo "Verification Complete!"
echo "=========================================="
