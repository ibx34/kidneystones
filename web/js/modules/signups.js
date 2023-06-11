export const showSignupDialog = (event) => {
    const signup_dialog = document.getElementById("signupDialog");
    signup_dialog.showModal();

    event.preventDefault();
}

export const closeSignupDialog = (checkIfOpen) => {
    const signup_dialog = document.getElementById("signupDialog");
    if (checkIfOpen && !signup_dialog.open) {
        return null;
    }
    signup_dialog.close();
}