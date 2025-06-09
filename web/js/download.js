document.addEventListener("DOMContentLoaded", function () {
  const downloadBtn = document.getElementById("downloadButton");
  const progressBar = document.getElementById("downloadProgress");
  const data = document.getElementById("data");

  if (!downloadBtn) return;

  downloadBtn.addEventListener("click", function () {
    progressBar.value = 0;
    progressBar.style.display = "block";

    fetch(data.dataset.download)
      .then((response) => {
        if (!response.ok) throw new Error("Network response was not ok");
        const contentLength = response.headers.get("content-length");
        if (!contentLength)
          throw new Error("Content-Length response header unavailable");
        const total = parseInt(contentLength, 10);
        let loaded = 0;

        return new Response(
          new ReadableStream({
            start(controller) {
              const reader = response.body.getReader();
              function read() {
                reader.read().then(({ done, value }) => {
                  if (done) {
                    controller.close();
                    return;
                  }
                  loaded += value.length;
                  const percent = Math.round((loaded / total) * 100);
                  progressBar.value = percent;
                  controller.enqueue(value);
                  read();
                });
              }
              read();
            },
          }),
        );
      })
      .then((res) => res.blob())
      .then((blob) => {
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = data.dataset.download.split("/").pop();
        document.body.appendChild(a);
        a.click();
        a.remove();
      })
      .catch((err) => {
        console.log(err);
        progressBar.style.display = "none";
      });
  });
});
