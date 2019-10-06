podTemplate(yaml:"""
apiVersion: v1
kind: Pod
spec:
    containers:
      - name: build
        image: jscheel42/build
        command:
            - cat
        tty: true
        resources:
            requests:
                cpu: 500m
                memory: 1024Mi
"""
) {
    node(POD_LABEL) {
        stage('build') {
            container('build') {
                steps {
                    withCredentials([string(credentialsId: 'dockerhub_password', variable: 'DOCKERHUB_PASSWORD')]) {
                        git url: 'https://github.com/jscheel42/little-lookup.git'
                        sh 'docker login --username jscheel42 --password $DOCKERHUB_PASSWORD'
                        sh 'bash build.sh push'
                    }
                }
            }
        }
    }
}
