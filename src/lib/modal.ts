export interface PromptOptions {
  title: string;
  defaultValue?: string;
  okLabel: string;
  cancelLabel: string;
}

export interface ConfirmOptions {
  title: string;
  okLabel: string;
  cancelLabel: string;
}

export function pixelPrompt(opts: PromptOptions): Promise<string | null> {
  return new Promise(resolve => {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    overlay.innerHTML = `
      <div class="modal-box">
        <h3>${escapeHtml(opts.title)}</h3>
        <input class="set-val modal-input" value="${escapeHtml(opts.defaultValue ?? "")}" />
        <div class="modal-btns">
          <button class="pixel-btn off" data-act="cancel">${escapeHtml(opts.cancelLabel)}</button>
          <button class="pixel-btn" data-act="ok">${escapeHtml(opts.okLabel)}</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    const input = overlay.querySelector<HTMLInputElement>(".modal-input")!;
    input.focus(); input.select();
    const cleanup = () => overlay.remove();
    const ok = () => { resolve(input.value); cleanup(); };
    const cancel = () => { resolve(null); cleanup(); };
    overlay.addEventListener("click", e => {
      const t = e.target as HTMLElement;
      if (t.dataset.act === "ok") ok();
      else if (t.dataset.act === "cancel") cancel();
      else if (t === overlay) cancel();
    });
    input.addEventListener("keydown", e => {
      if (e.key === "Enter") { e.preventDefault(); ok(); }
      else if (e.key === "Escape") { e.preventDefault(); cancel(); }
    });
  });
}

export function pixelConfirm(opts: ConfirmOptions): Promise<boolean> {
  return new Promise(resolve => {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    overlay.innerHTML = `
      <div class="modal-box">
        <h3>${escapeHtml(opts.title)}</h3>
        <div class="modal-btns">
          <button class="pixel-btn off" data-act="cancel">${escapeHtml(opts.cancelLabel)}</button>
          <button class="pixel-btn" data-act="ok">${escapeHtml(opts.okLabel)}</button>
        </div>
      </div>`;
    document.body.appendChild(overlay);
    overlay.querySelector<HTMLButtonElement>('[data-act="ok"]')!.focus();
    const cleanup = () => overlay.remove();
    const yes = () => { resolve(true); cleanup(); };
    const no = () => { resolve(false); cleanup(); };
    overlay.addEventListener("click", e => {
      const t = e.target as HTMLElement;
      if (t.dataset.act === "ok") yes();
      else if (t.dataset.act === "cancel") no();
      else if (t === overlay) no();
    });
    overlay.addEventListener("keydown", e => {
      if (e.key === "Enter") { e.preventDefault(); yes(); }
      else if (e.key === "Escape") { e.preventDefault(); no(); }
    });
  });
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, c =>
    ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;" }[c]!)
  );
}
