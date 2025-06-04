document
  .getElementById("uploadForm")
  .addEventListener("submit", async function (event) {
    event.preventDefault();
    const form = event.target;
    const formData = new FormData(form);
    const response = await fetch(form.action, {
      method: form.method,
      body: formData,
    });
    if (response.ok) {
      const data = await response.json();
      console.log(data);
      window.location.href = `${window.location.href}file/${data.id}`;
    } else {
      alert("Upload failed.");
    }
  });
