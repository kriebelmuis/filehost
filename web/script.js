document
  .getElementById("uploadForm")
  .addEventListener("submit", function (event) {
    event.preventDefault();
    const form = event.target;
    const formData = new FormData(form);

    // Show progress bar
    let progressBar = document.getElementById("progressBar");
    if (!progressBar) {
      progressBar = document.createElement("progress");
      progressBar.id = "progressBar";
      progressBar.value = 0;
      progressBar.max = 100;
      form.appendChild(progressBar);
    }
    progressBar.value = 0;
    progressBar.style.display = "block";

    const xhr = new XMLHttpRequest();
    xhr.open(form.method, form.action, true);

    xhr.upload.onprogress = function (e) {
      if (e.lengthComputable) {
        const percentComplete = (e.loaded / e.total) * 100;
        progressBar.value = percentComplete;
      }
    };

    xhr.onload = function () {
      if (xhr.status === 200) {
        const data = JSON.parse(xhr.responseText);
        window.location.href = `${window.location.href}file/${data.id}.${data.ext}`;
      } else {
        alert("Upload failed.");
      }
      progressBar.style.display = "none";
    };

    xhr.onerror = function () {
      alert("Upload failed.");
      progressBar.style.display = "none";
    };

    xhr.send(formData);
  });
