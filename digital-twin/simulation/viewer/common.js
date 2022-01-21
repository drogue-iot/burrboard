
let state = {
    connected: false,
};

function setState(accel) {
    if (accel) {
        setField('accel-x', accel.x);
        setField('accel-y', accel.y);
        setField('accel-z', accel.z);
        state.accel = accel;
    } else {
        setField('accel-x', '?');
        setField('accel-y', '?');
        setField('accel-z', '?');
        state.accel = null;
    }
}

function setField(id, value) {
    let field = document.getElementById(id);
    field.innerText = typeof value == 'number' && value.toFixed(1) || value;
}
