import init, * as wasm from "./pkg/interface_backend.js";

const rawContainer = document.getElementById('raw-data-container');

// cache the field -> element lookup once, instead of re-querying every message
const fieldEls = {};
rawContainer.querySelectorAll('.value[data-field]').forEach(el => {
    fieldEls[el.dataset.field] = el;
});

function setField(field, value) {
    const el = fieldEls[field];
    if (el) el.textContent = value;
}

const message_buffer = [];
      const ctx = document.getElementById('mychart');
const chart = new Chart(ctx, {
        type: 'line',
        data: {
            datasets: [{
                // data: message_buffer.map((e) => {
                //     return {x: e.elapsed_time, y: e.voltage}
                // }),
                data: [
                    {x: 1, y: 1.0},
                    {x: 1000, y: 2.0},
                    {x: 2000, y: 3.0},
                    {x: 3000, y: 4.0},
                    {x: 4000, y: 5.0},
                ],
                borderWidth: 1,
            }]
        },
        options: {
            animation: false,
            parsing: false,
            scales: {
                x: {
                    type: 'linear',
                    // min: 0,
                    // max: 10000
                },
                y: {
                    beginAtZero: true
                }
            }
        }
    });

function send_message(new_packet) {
    msg_packet = new_packet;
    message_buffer.push(new_packet);
    // console.log(message_buffer)
    if (message_buffer.length > 500) {
        message_buffer.shift()
    }

    chart.data.datasets[0] = {
        data: message_buffer.map((e) => {
            return {x: e.elapsed_time, y: e.voltage}
        }),
        borderWidth: 1
    };
    chart.update()

    const fc_power = new_packet.fc_voltage * new_packet.fc_current;

    setField('voltage', new_packet.voltage.toFixed(2));
    setField('current', new_packet.current.toFixed(2));
    setField('power', new_packet.power.toFixed(2));
    setField('batt_flags', new_packet.batt_flags);
    setField('fc_voltage', new_packet.fc_voltage.toFixed(2));
    setField('fc_current', new_packet.fc_current.toFixed(2));
    setField('fc_power', fc_power.toFixed(2));
    setField('fc_flags', new_packet.fc_flags);
    setField('energy_wh', new_packet.energy_wh.toFixed(3));
    setField('hydro_wh', new_packet.hydro_wh.toFixed(3));
    setField('elapsed_time', new_packet.elapsed_time);
    setField('time_since_purge', new_packet.time_since_purge);
    setField('free_heap', new_packet.free_heap);
    setField('detected_zero', new_packet.detected_zero);
    setField('current_cal', new_packet.current_cal);
    setField('acc', Array.from(new_packet.acc).map(v => v.toFixed(2)).join(', '));
    setField('gyr', Array.from(new_packet.gyr).map(v => v.toFixed(2)).join(', '));
    setField('temp', new_packet.temp.toFixed(1));
}

function send_scan(scan) {
    const container = document.createElement('div');
    container.className = "scanned-brain";
    const title = document.createElement('span');
    title.className = "brain-name";
    title.innerHTML = scan.name;
    container.appendChild(title);
    console.log("got scan", scan);
    const macAddress = Array.from(scan.mac_address)
        .map(b => b.toString(16).padStart(2, '0'))
        .join(':');
    const mac_label = document.createElement('span');
    mac_label.className = "brain-mac";
    mac_label.innerHTML = macAddress;
    container.appendChild(mac_label);

    const connect_button = document.createElement('button');
    connect_button.className = "brain-connect";
    connect_button.innerHTML = "connect";
    container.appendChild(connect_button);

    // Elements for the "connected" state, created once and reused
    const disconnect_button = document.createElement('button');
    disconnect_button.className = "brain-disconnect";
    disconnect_button.innerHTML = "disconnect";

    const rename_button = document.createElement('button');
    rename_button.className = "brain-rename";
    rename_button.innerHTML = "rename";

    const rename_input = document.createElement('input');
    rename_input.type = "text";
    rename_input.className = "brain-rename-input";
    rename_input.placeholder = "new name";

    function showConnectedState() {
        connect_button.remove();
        container.appendChild(disconnect_button);
        container.appendChild(rename_button);
        container.appendChild(rename_input);
        document.getElementById("wireless").style.visibility = 'hidden';
    }

    function showDisconnectedState() {
        disconnect_button.remove();
        rename_button.remove();
        rename_input.remove();
        document.getElementById("wireless").style.visibility = 'visible';
        container.appendChild(connect_button);
    }

    connect_button.addEventListener("click", async  (e) => {
        console.log(scan.mac_address);
        await wasm.connect_to_brain(scan.mac_address);
        showConnectedState();
    });

    disconnect_button.addEventListener("click", async (e) => {
        await wasm.disconnect_brain();
        // if you have a wasm disconnect call, invoke it here, e.g.:
        // wasm.disconnect_from_brain();
        showDisconnectedState();
    });

    rename_button.addEventListener("click", async (e) => {
        const newName = rename_input.value.trim();
        if (newName) {
            title.innerHTML = newName;
            rename_input.value = "";
        }
        await wasm.rename_brain(newName);
    });

    document.querySelector("#wireless > .connect-container").appendChild(container);
    scanned_brains.push(container);
}

window.send_message = send_message;
window.send_scan = send_scan;

let scanned_brains = [];

async function run() {
    await init();
    document.getElementById("connect").addEventListener("click", async () => {
        await wasm.connect();
        document.getElementById("serial").style.visibility = 'hidden';
        document.getElementById("wireless").style.visibility = 'visible';
        document.getElementById("scan").click();
    });
    document.getElementById("rerec").addEventListener("click", () => {
        wasm.restart_receiver();
    });
    document.getElementById("scan").addEventListener("click", () => {
        for (let i of scanned_brains) {
            document.querySelector("#wireless > .connect-container").removeChild(i);
        }
        scanned_brains = []
        wasm.write_bytes([1])
    });
    document.getElementById("debug").addEventListener("change", (e) => {
        console.log(e.target.value);
        wasm.set_debug(e.target.value == "on");
    });
    window.msg_packet = new wasm.JSMessagePacket();
    console.log(msg_packet.voltage);
}

run();