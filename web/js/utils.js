export const copyTextToClipboard = (event, text) => {
    event.preventDefault();
    const btn = event.target;
    let btn_original_value = btn.innerHTML;
    navigator.clipboard.writeText(text).catch(() => {
        btn.innerHTML = "failed to copy";
        setTimeout(() => {
            btn.innerHTML = btn_original_value;
        }, 2000);
    }).then(() => {
        btn.innerHTML = "copied!";
        setTimeout(() => {
            btn.innerHTML = btn_original_value;
        }, 2000);
    })
}