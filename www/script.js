const table_title = [
  "bus",
  "dev",
  "ch",
  "input (V)",
  "Voltage (V)",
  "Current (uA)",
  "about",
];
const mhv4_discription = [
  "long long long long long long long long long long",
  "1-2",
  "1-3",
  "1-4",
  "2-1",
  "2-2",
  "2-3",
  "2-4",
];

async function DisplayData() {
  try {
    const response = await fetch("/mhv4_data");
    if (!response.ok) {
      throw new Error("failed to fetch the MHV4 data");
    }
    const data = await response.json();
    console.log(data);
    setStatus(data);
    createTable(data);
  } catch (error) {
    console.error("Error:", error);
  }
}

function setStatus(data) {
  const obj = JSON.parse(data);
  const circle1 = document.getElementById("statusCircle1");
  if (obj.is_rc) {
    circle1.className = "status-circle green";
  } else {
    circle1.className = "status-circle red";
  }

  const circle2 = document.getElementById("statusCircle2");
  if (obj.is_on) {
    circle2.className = "status-circle green";
  } else {
    circle2.className = "status-circle red";
  }
}

function createTable(data) {
  const container = document.getElementById("TableContainer");
  const table = document.createElement("table");
  table.setAttribute("border", "1");
  table.style.border = "2px solid green";

  const thead = document.createElement("thead");
  const headerRow = document.createElement("tr");
  table_title.forEach((field, index) => {
    const th = document.createElement("th");
    th.textContent = field;
    headerRow.appendChild(th);
  });
  thead.appendChild(headerRow);
  table.appendChild(thead);

  // table data
  const tbody = document.createElement("tbody");
  const obj = JSON.parse(data);
  obj["mhv4_data_array"].forEach((mod, index) => {
    const row = document.createElement("tr");
    row.appendChild(createCell(mod.bus));
    row.appendChild(createCell(mod.dev));
    row.appendChild(createCell(mod.ch));

    // input field
    const inputCell = document.createElement("td");
    const input = document.createElement("input");
    input.type = "number";
    input.value = (Math.abs(mod.current) * 0.1).toFixed(1);
    input.step = 0.1;
    input.min = 0;
    inputCell.appendChild(input);
    row.appendChild(inputCell);

    // SSE field (initially empty)
    row.appendChild(createCell(""));
    row.appendChild(createCell(""));

    row.appendChild(createCell(mhv4_discription[index]));

    tbody.appendChild(row);
  });

  table.appendChild(tbody);
  container.appendChild(table);
}

function createCell(text) {
  const cell = document.createElement("td");
  const celltext = document.createTextNode(text);
  cell.appendChild(celltext);
  return cell;
}

function setupSSE() {
  const eventSource = new EventSource("/sse");
  eventSource.onopen = function (event) {
    console.log("SSE connection opened:", event);
  };
  eventSource.onmessage = function (event) {
    console.log("SSE message received:", event);
    const data = JSON.parse(event.data);
    updateTable(data);
    animateCell();
  };
  eventSource.onerror = function (error) {
    console.error("SSE connection opened:", error);
  };
}

function updateTable(data) {
  const table = document.querySelector("table");
  if (data[2]) {
    table.style.border = "2px solid yellow";
  } else {
    table.style.border = "2px solid green";
  }

  for (let i = 0; i < table.rows.length - 1; i++) {
    const row = table.rows[i + 1];
    const v_cell = row.cells[4];
    const c_cell = row.cells[5];

    if (data[0][i] < -99999) {
      v_cell.textContent = "read error!";
    } else {
      v_cell.textContent = (data[0][i] * 0.1).toFixed(1);
    }
    if (data[1][i] < -99999) {
      c_cell.textContent = "read error!";
    } else {
      c_cell.textContent = (data[1][i] * 0.001).toFixed(3);
    }
  }
}

function animateCell() {
  const table = document.querySelector("table");
  const v_cell = table.rows[0].cells[4];
  const c_cell = table.rows[0].cells[5];
  [v_cell, c_cell].forEach((cell) => {
    cell.style.backgroundColor = "yellow";
    setTimeout(() => {
      cell.style.backgroundColor = "#e9faf9";
    }, 200);
  });
}

// 0: RC on, 1: RC off, 2: Power on, 3: Power off
async function ChangeStatus(num) {
  const circle1 = document.getElementById("statusCircle1");
  const circle2 = document.getElementById("statusCircle2");
  if (num == 0 || num == 1) {
    circle1.className = "status-circle yellow";
  } else {
    circle2.className = "status-circle yellow";
  }

  const response = await fetch("/status", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(num),
  });
  const result = await response.json();
  if (result) {
    if (num == 0) {
      console.log("set RC ON");
      circle1.className = "status-circle green";
    } else if (num == 1) {
      console.log("set RC OFF");
      circle1.className = "status-circle red";
    } else if (num == 2) {
      console.log("set Power ON");
      circle2.className = "status-circle green";
    } else {
      console.log("set Power OFF");
      circle2.className = "status-circle red";
    }
  }
}

async function ApplyHV() {
  const table = document.querySelector("table");
  table.style.border = "2px solid yellow";

  try {
    const rows = table.querySelectorAll("tr");
    let data = [];
    rows.forEach((row, index) => {
      if (index == 0) return;
      const input = row.cells[3].querySelector("input");
      if (input.value == "" || Number(input.value) < 0) {
        throw new Error("failed to fetch the MHV4 data");
      }
      const value = parseInt(Number(input.value) * 10);
      data.push(value);
    });
    console.log("send HV data", data);

    const response = await fetch("/apply", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    });
    const result = await response.json();
  } catch (error) {
    table.style.border = "2px solid red";
    console.error("Input Error:", error);
  }
}

document.addEventListener("DOMContentLoaded", () => {
  DisplayData();
  setupSSE();
});

function Time() {
  const realTime = new Date();
  const year = realTime.getFullYear();
  const month = realTime.getMonth() + 1;
  const day = realTime.getDate();
  const hour = realTime.getHours();
  const minutes = realTime.getMinutes();
  const seconds = realTime.getSeconds();
  const text =
    year +
    "/" +
    ("00" + month).slice(-2) +
    "/" +
    ("00" + day).slice(-2) +
    " " +
    ("00" + hour).slice(-2) +
    ":" +
    ("00" + minutes).slice(-2) +
    ":" +
    ("00" + seconds).slice(-2);
  document.getElementById("real_time").innerHTML = text;
}
setInterval("Time()", 1000);
