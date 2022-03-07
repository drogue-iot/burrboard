
To build the image

    $ docker build -t drogue-gateway deploy/

To run a container

    $ docker run --net=host --privileged -it drogue-gateway
	
Valid tokens can be found grep'ing for "token" beneath mesh/
