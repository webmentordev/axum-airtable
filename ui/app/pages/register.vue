<template>
    <div class="w-full h-screen">
        <div class="w-full h-full flex items-center justify-center">
            <div class="flex flex-col max-w-87.5 w-full">
                <div class="flex items-center m-auto">
                    <img src="/wind.png" alt="AT Icon" width="70">
                    <strong class="text-gray-600 font-medium text-3xl ml-2">Windtable</strong>
                </div>
                <p class="text-center text-gray-600 my-5">Create windtable account</p>
                <form @submit.prevent="login" method="post">
                    <div class="grid grid-cols-1 gap-3">
                        <div class="flex flex-col">
                            <AppInput v-model="name" type="text" placeholder="Name" />
                            <AlertsAlertError v-if="errors.name" error="Name field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="username" type="text" placeholder="Username" />
                            <AlertsAlertError v-if="errors.username" error="Username field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="email" type="email" placeholder="Email address" />
                            <AlertsAlertError v-if="errors.email" error="Email field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="password" type="password" placeholder="Password" />
                            <AlertsAlertError v-if="errors.password" error="Password field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="confirm_password" type="password" placeholder="Confirm password" />
                            <AlertsAlertError v-if="errors.confirm_password"
                                error="Confirm password field is required" />
                        </div>
                    </div>
                    <button v-if="!processing" type="submit"
                        class="bg-main mt-4 text-white w-full py-3 rounded-xl flex items-center justify-center hover:bg-main/90 group">
                        <span class="mr-3">Register</span>
                        <img class="mt-1 transition-all group-hover:transition-all group-hover:translate-x-4"
                            src="https://api.iconify.design/line-md:arrow-right.svg?color=%23ffffff" width="15">
                    </button>


                    <p class="text-para-light inline-block text-sm ml-1 mt-3">Already have an account? <NuxtLink
                            to="/login" class="text-main underline">Login here</NuxtLink>
                    </p>

                    <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
                    <AppLoading v-if="processing" message="Processing signup request..." />
                    <AlertsError v-if="errors.message" :message="errors.message" />
                </form>
            </div>
        </div>
    </div>
</template>

<script setup>
definePageMeta({
    middleware: 'guest'
});

const name = ref("");
const username = ref("");
const email = ref("");
const password = ref("");
const confirm_password = ref("");
const processing = ref(false);
const message = ref(false);
const errors = ref({
    count: 0
});

async function login() {
    processing.value = true;
    message.value = false;
    reset_errors();
    if (name.value == "") {
        errors.value.name = "Name is required";
        errors.value.count += 1;
    }
    if (username.value == "") {
        errors.value.username = "Username is required";
        errors.value.count += 1;
    }
    if (email.value == "") {
        errors.value.email = "Email is required";
        errors.value.count += 1;
    }
    if (password.value == "") {
        errors.value.password = "Password is required";
        errors.value.count += 1;
    }
    if (confirm_password.value == "") {
        errors.value.confirm_password = "Password confirmation is required";
        errors.value.count += 1;
    }
    if (password.value != confirm_password.value) {
        errors.value.password = "Password and Confirm password is required";
        errors.value.count += 1;
    }
    if (errors.value.count > 0) {
        processing.value = false;
        return;
    };
    try {
        const data = await $fetch("/api/auth/register", {
            method: "POST",
            body: {
                name: name.value.trim(),
                username: username.value.trim(),
                email: email.value.trim(),
                password: password.value.trim(),
                confirm_password: confirm_password.value.trim(),
            }
        });
        if (data.message) {
            message.value = data.message;
            reset_values();
        }
    } catch (e) {
        errors.value.message = e.statusMessage || 'Something went wrong!';
    } finally {
        processing.value = false;
    }
}

function reset_errors() {
    errors.value = {
        count: 0
    };
}

function reset_values() {
    name.value = "";
    username.value = "";
    email.value = "";
    password.value = "";
    confirm_password.value = "";
}

</script>