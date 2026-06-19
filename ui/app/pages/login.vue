<template>
    <div class="w-full h-screen">
        <div class="w-full h-full flex items-center justify-center">
            <div class="flex flex-col max-w-87.5 w-full">
                <div class="flex items-center m-auto">
                    <img src="/wind.png" alt="AT Icon" width="70">
                    <strong class="text-gray-600 font-medium text-3xl ml-2">Windtable</strong>
                </div>
                <p class="text-center text-gray-600 my-5">Account Login</p>
                <form @submit.prevent="login" method="post">
                    <div class="grid grid-cols-1 gap-3">
                        <div class="flex flex-col">
                            <AppInput v-model="email" type="email" placeholder="Email address" />
                            <AlertsAlertError v-if="errors.email" error="Email field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="password" type="password" placeholder="Password" />
                            <AlertsAlertError v-if="errors.password" error="Password field is required" />
                        </div>
                    </div>

                    <NuxtLink to="/forgot-password"
                        class="text-para-light inline-block underline text-sm ml-1 mt-3 mb-3 hover:text-main">Forgotten
                        password?</NuxtLink>

                    <Button v-if="!processing" type="submit" text="Login" />

                    <p class="text-para-light inline-block text-sm ml-1 mt-3">Don't have an account? <NuxtLink
                            to="/register" class="text-main underline">Register here</NuxtLink>
                    </p>

                    <AppLoading v-if="processing" message="Processing login request..." />
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

const { setToken } = useAuthToken();
const email = ref("");
const password = ref("");
const processing = ref(false);
const errors = ref({
    email: null,
    password: null,
    message: null,
    count: 0
});

async function login() {
    processing.value = true;
    reset_errors();
    if (email.value == "") {
        errors.value.email = "Email is required";
        errors.value.count += 1;
    }
    if (password.value == "") {
        errors.value.password = "Password is required";
        errors.value.count += 1;
    }
    if (errors.value.count > 0) {
        processing.value = false;
        return;
    };
    try {
        const data = await $fetch("/api/auth/login", {
            method: "POST",
            body: {
                email: email.value.trim(),
                password: password.value.trim()
            }
        });
        if (data.token) {
            setToken(data.token);
            reset_values();
            await navigateTo('/');
        }
    } catch (e) {
        errors.value.message = e.statusMessage || 'Invalid login credientials.';
    } finally {
        processing.value = false;
    }
}

function reset_errors() {
    errors.value = {
        email: null,
        password: null,
        message: null,
        count: 0
    };
}

function reset_values() {
    email.value = "";
    password.value = "";
}

</script>