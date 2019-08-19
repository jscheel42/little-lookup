pipeline {  
    agent any  
    stages {  
        stage('Build') {
            steps {
                withCredentials([string(credentialsId: 'dockerhub_password', variable: 'DOCKERHUB_PASSWORD')]) {
                    git url: 'https://github.com/jscheel42/docker-build.git'
                    sh 'docker login --username jscheel42 --password $DOCKERHUB_PASSWORD'
                    sh 'bash build.sh push'
                }
            }
        }
    }  
    post {  
        failure {  
            emailext subject: "${env.JOB_NAME} build #${env.BUILD_NUMBER} failed",
                body: "${env.BUILD_URL}",
                replyTo: 'jscheel42@gmail.com',
                to: 'jscheel42@gmail.com' 
        }  
    }  
}