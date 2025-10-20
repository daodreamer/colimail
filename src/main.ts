import { invoke } from "@tauri-apps/api/core";

document.getElementById("btn")!.addEventListener("click", async () => {
  const result = await invoke<string>("get_status");
  document.getElementById("output")!.innerText = result;
});
