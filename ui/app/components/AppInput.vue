<template>
    <input :value="props.modelValue" @input="emit('update:modelValue', $event.target.value)" autocomplete="off"
        :class="[$attrs.class, 'p-3 border border-gray-200 bg-gray-100 outline-none placeholder:text-gray-500 focus:outline-none w-full text-gray-600 text-sm rounded-xl']"
        v-bind="{ ...$attrs, class: undefined }">
</template>

<script setup>
const props = defineProps(['modelValue'])
const emit = defineEmits(['update:modelValue'])
</script>