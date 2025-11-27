const useLocalCache = () => {
    const save_bookmarks = (json_string: string) => {
        localStorage.setItem('bookmarks', json_string);
    }

    const load_bookmarks = () => {
        return localStorage.getItem('bookmarks') || '';
    }

    return {save_bookmarks, load_bookmarks};
}

export {
    useLocalCache
}