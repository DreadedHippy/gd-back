echo "Deploying backend..."

set -o allexport
source .env set

docker build --build-arg postgres_admin_name=$POSTGRES_ADMIN_NAME --build-arg database_url=$DATABASE_URL --build-arg postgres_admin_pass=$POSTGRES_ADMIN_PASS -t $DOCKER_IMAGE_NAME .

if [ $? -eq 0 ]; then
  echo "Image built successfully"
	echo "Pushing to cloud..."
	docker push $DOCKER_IMAGE_NAME

	if [ $? -eq 0 ]; then
		echo "Image pushed successfully"
	else
		echo "Image push failed"
	fi

else
  echo "Command Failed"
fi