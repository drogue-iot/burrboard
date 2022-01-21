function render() {
    const MathUtils = THREE.MathUtils;
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);

    const renderer = new THREE.WebGLRenderer();
    const canvas = document.getElementById("canvas");
    console.log("Changed ", canvas.offsetWidth, canvas.offsetHeight);
    renderer.setSize(canvas.offsetWidth, canvas.offsetHeight);
    canvas.onresize = function(){
        console.log("Changed ", canvas.offsetWidth, canvas.offsetHeight);
        renderer.setSize(canvas.offsetWidth, canvas.offsetHeight);
    };
    canvas.appendChild(renderer.domElement);

    const light = new THREE.AmbientLight(0x808080); // soft white light
    scene.add(light);
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5);
    scene.add(directionalLight);

    const geometry = new THREE.BoxGeometry(2, 4, .2);
    const materialDisconnected = new THREE.MeshStandardMaterial({color: 0xff0000});
    const materialConnected = new THREE.MeshStandardMaterial({color: 0x00ff00});
    const cube = new THREE.Mesh(geometry, materialDisconnected);
    scene.add(cube);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);

        if (state.connected) {
            cube.material = materialConnected;
        } else {
            cube.material = materialDisconnected;
        }

        if (state.accel) {
            cube.rotation.x = MathUtils.degToRad(state.accel.x);
            cube.rotation.y = MathUtils.degToRad(state.accel.y);
            cube.rotation.z = MathUtils.degToRad(state.accel.z);
        }

        renderer.render(scene, camera);
    }

    animate();
}
