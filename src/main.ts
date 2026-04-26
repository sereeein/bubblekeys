import { renderHome } from "./views/home";

const screen = document.getElementById("screen")!;
renderHome(screen).catch(err => {
  screen.textContent = String(err);
});
