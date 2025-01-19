kubectl delete svc svc-nilspetrust-psql
kubectl delete svc svc-nilspetrust-redis
kubectl delete deployment dpl-nilspetrust-psql
kubectl delete deployment dpl-nilspetrust-redis

SCRIPT_DIR=$(pwd)
echo $SCRIPT_DIR
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
echo $PROJECT_DIR

cd $PROJECT_DIR

docker rmi --force $(docker images -q 'nilspetrust-redis' | uniq)
docker rmi --force $(docker images -q 'nilspetrust-postgresql' | uniq)

minikube image rm --force 'nilspetrust-redis'
minikube image rm --force 'nilspetrust-postgresql'

cd $SCRIPT_DIR/postgresql
docker build -t nilspetrust-postgresql:latest .

cd $SCRIPT_DIR/redis
docker build -t nilspetrust-redis:latest .

eval $(minikube docker-env)

minikube image load 'nilspetrust-redis'
minikube image load 'nilspetrust-postgresql'

cd $SCRIPT_DIR
kubectl apply -f $SCRIPT_DIR/postgresql/dpl-psql.yaml --validate=false
kubectl apply -f $SCRIPT_DIR/postgresql/svc-psql.yaml
kubectl apply -f $SCRIPT_DIR/redis/dpl-redis.yaml --validate=false
kubectl apply -f $SCRIPT_DIR/redis/svc-redis.yaml