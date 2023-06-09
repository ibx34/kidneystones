export function submitCreateRepoForm(event, logged_in) {
    const create_repo_form_data = new FormData(event.target);
    let new_repo_name = create_repo_form_data.get("new-repo-name");
    if (new_repo_name == null) {
        alert("Name may not be empty.");
        event.preventDefault();
        return;
    }
    let name_len = new_repo_name.length;
    if (name_len > 32 || name_len < 1) {
        alert(`Your name may be too long, or too short. It may not be longer than 32 characters and must be longer than 1 character. (${name_len}/32)`);
        event.preventDefault();
        return;
    }

    if (!logged_in && create_repo_form_data.get("not-logged-in-user-agreement") === null) {
        alert(`You must agree to the Terms of Service, Privacy Policy and Anonymous Repository policy before creating a new repo.`);
        event.preventDefault();
        return;
    }
    let new_form_object = {
        name: new_repo_name,
        private: create_repo_form_data.get("new-repo-is-private") === null ? false : create_repo_form_data.get("new-repo-is-private") == "on",
    };
    alert(JSON.stringify(new_form_object));
    fetch('/api/repos/create', {
        method: 'post',
        body: JSON.stringify(new_form_object),
        headers: { 'Content-Type': 'application/json' }
    }).catch((err) => {
        alert(`Failed to create repo: ${err.body}`)
    }).then((res) => {
        alert(`New repo: ${JSON.stringify(res.json)}`)
    });
    event.preventDefault();
}