mergeInto(LibraryManager.library, {
    'ruggle_sync_idbfs': function () {
        FS.syncfs(false, (err) => {});
    },
});
